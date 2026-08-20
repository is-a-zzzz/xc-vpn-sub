# 获取官方客户端订阅请求头(UA)的完整过程

> 目标:星辰VPN(xcvpn.us)服务端按 User-Agent 识别客户端类型并决定返回内容。本文记录如何从官方客户端安装包中逆向出它请求订阅时使用的 UA,并验证生效。全部步骤可在 macOS 上复现,Windows/Android 包同理。

## 背景:为什么要找这个 UA

同一个订阅接口,不同 UA 的返回差异巨大:

| 请求方 UA | 返回结果 |
|---|---|
| 普通浏览器 | 406 Not Acceptable,直接拒绝 |
| ClashforWindows/0.20.39 | 200,但只有 3 个节点(2 真实 + 1 个"请下载新客户端"占位) |
| **官方客户端的 UA** | 200,**完整 44 个节点**的 YAML 配置 |
| v2rayN 等其他客户端标识 | 200,但对应格式(base64 节点列表)的响应在部分网络链路上会被中途掐断 |

所以:要么找到官方客户端的 UA,要么只能拿到残缺节点列表。

## 第一步:下载官方客户端

登录面板(如 `https://xcvpn.us/dashboard`),在"客户端下载"区拿到底链,形如:

```sh
curl -O "https://download.kilxs.cn/xingchen/Xingchen-<版本>-macos.zip"
unzip Xingchen-<版本>-macos.zip -d xingchen
```

macos.zip 里是三个架构的 DMG(arm64/amd64/amd64-compatible)加一份使用教程 md。教程本身有信息量:应用名、Bundle ID、TUN 权限要求等。

## 第二步:挂载 DMG,看应用结构

```sh
hdiutil attach -nobrowse -readonly Xingchen-<版本>-macos-arm64.dmg
APP=/Volumes/XingchenVPN/XingchenVPN.app

# 注册的 URL scheme —— 第一个线索
plutil -p "$APP/Contents/Info.plist" | grep -A 8 CFBundleURLSchemes
```

典型输出会看到 `clash`、`clashmeta` 等_scheme_——强烈暗示内核是 Clash Meta 系。

再看可执行文件:

```sh
ls -la "$APP/Contents/MacOS/"
# XingchenVPN   → 主程序(Flutter 壳,几百 KB)
# XingchenCore  → 核心二进制(几十 MB,真正干活的)
file "$APP/Contents/MacOS/XingchenCore"   # Mach-O 64-bit executable arm64
```

## 第三步:确认内核身份

对核心二进制做字符串搜索:

```sh
strings -a "$APP/Contents/MacOS/XingchenCore" | grep -i mihomo | head
# 大量 github.com/metacubex/mihomo/... → 内核是 mihomo(Clash Meta)
```

这一步确定:客户端能解析的节点协议 = mihomo 支持的全集(anytls/trojan/ss/vmess/vless/hysteria2/tuic 等)。

## 第四步:找 UA 字符串(关键步骤)

界面层的 Dart 代码编译在 `App.framework` 里,字符串明文可见:

```sh
APPA="$APP/Contents/Frameworks/App.framework/Versions/A/App"

# 4.1 先看品牌配置(私有 API 地址、远程配置域名都在这)
cat "$APP/Contents/Frameworks/App.framework/Versions/A/Resources/flutter_assets/assets/app_config.json"
# 关注字段: apiBaseUrls / remoteConfigUrls / appName

# 4.2 搜 API 路径,确认接口体系(V2Board 风格)
strings -a "$APPA" | grep -E "^/(guest|passport|user)/" | sort -u
# /passport/auth/login、/user/getSubscribe、/user/server/fetch ...

# 4.3 搜 UA —— 用框架代号、版本号特征去捞
strings -a "$APPA" | grep -iE "Bettbox|^[A-Za-z]+/[0-9]+\.[0-9]+" | sort -u
```

4.3 的关键产出(本次实际结果):

```
Clash/Meta/Mihomo/ClashMetaForAndroid/Bettbox/v2.11.22
bluebird/1.0.10,clash-meta/1.19.20
```

第一条就是取订阅用的 UA——一个串里塞了 clash/meta/mihomo/clashmetaforandroid 全部关键词,服务端无论按哪个关键词匹配都会识别为"自家客户端"。`Bettbox` 是客户端内部框架代号,`v2.11.22` 是其版本;发现 `/tmp/BettboxSocket_` 等字符串可以佐证。

## 第五步:验证

1. 面板 → 订阅链接 → 点"生成",得到一次性地址(`.../api/v1/client/secureSubscribe?token=...`),**1 分钟内有效,取一次即失效**
2. 立刻用逆向出的 UA 请求:

```sh
curl -s -A "Clash/Meta/Mihomo/ClashMetaForAndroid/Bettbox/v2.11.22" \
  -o full.yaml -w "HTTP:%{http_code} SIZE:%{size_download}\n" \
  "https://xcsuburl.kilxs.cn/api/v1/client/secureSubscribe?token=<一次性token>"

python3 -c "import yaml;print('节点数:',len(yaml.safe_load(open('full.yaml'))['proxies']))"
```

验证通过的标准:`HTTP:200`,节点数为完整列表(本文时点为 44),协议分布与官网宣传的节点地图吻合。

对比实验(可选,各需一个新 token):

```sh
# 普通客户端 UA → 只给 3 个节点
curl -s -A "ClashforWindows/0.20.39" ...
# 无特殊 UA → 406
curl -s -A "Mozilla/5.0" ...
```

## 注意事项

- **token 一次性**:每次验证都要重新在面板生成,过期/用完会返回 `{"message":"ticket or token expired"}`
- **UA 不是永久的**:客户端升级后框架版本号可能变化,重新执行第四步 4.3 即可重新拿到
- **响应格式与传输**:不同 UA 对应不同响应格式,base64 节点列表类格式在部分链路会被掐断(表现为 curl 000/挂起),YAML 格式(Clash 系 UA)则稳定——本项目 `SUB_USER_AGENT` 默认值选用官方客户端 UA 即为此故
- Windows 包(zip 里的 exe)与 Android APK 用同样思路:`strings` 搜框架代号与版本号特征即可,无需执行安装包
- 本项目已将此 UA 设为 `SUB_USER_AGENT` 的默认值(`src/config.rs` 中 `DEFAULT_SUB_USER_AGENT`),留空该变量则改为转发客户端请求自带的 UA

## 时间线摘要(本文实测)

1. Clash UA 取回订阅 → 发现只有 3 节点 + 占位提示"节点少请下载新客户端"
2. 猜官方客户端 UA(Xingchen/1.0.14 等)→ 406,猜不中
3. 下载 macOS 客户端 → URL scheme 与 strings 确认 mihomo 内核
4. `app_config.json` 拿到私有 API;strings 拿到 Bettbox UA
5. 面板生成新 token + Bettbox UA 请求 → 44 节点完整列表,验证闭环
