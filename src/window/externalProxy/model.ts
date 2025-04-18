// 代理类型枚举
enum ProxyType {
  SOCKS = "socks",
  HTTP = "http",
  HTTPS = "https"
}

// 代理配置项接口
interface MutableExternalProxyConfiguration {
  username: string;
  encryptedPassword: string;
}

interface Entry {
  string: string;
  mutableExternalProxyConfiguration: MutableExternalProxyConfiguration;
}

// 主接口
interface ExternalProxy {
  configurations: { entry: Entry[] };
  bypassDomains: {
    string: string[];
  };
  enabled: boolean;
  proxyType: ProxyType;
  alwaysBypassLocalhost: boolean;
}

const defaultData: ExternalProxy = {
  configurations: {
    entry: [
      {
        string: ProxyType.HTTP,
        mutableExternalProxyConfiguration: {
          username: "",
          encryptedPassword: ""
        }
      },
      {
        string: ProxyType.HTTPS,
        mutableExternalProxyConfiguration: {
          username: "",
          encryptedPassword: ""
        }
      },
      {
        string: ProxyType.SOCKS,
        mutableExternalProxyConfiguration: {
          username: "",
          encryptedPassword: ""
        }
      }
    ]
  },
  bypassDomains: {
    string: []
  },
  proxyType: ProxyType.HTTP,
  enabled: false,
  alwaysBypassLocalhost: true
};

export type { MutableExternalProxyConfiguration, Entry, ExternalProxy };
export { ProxyType, defaultData };
