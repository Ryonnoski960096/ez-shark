// crypto.ts
import CryptoJS from "crypto-js";

// 定义配置接口
interface CryptoConfig {
  readonly SECRET_KEY: string;
  readonly IV: string;
}

//AES-256-CBC

// 定义加密选项接口
interface CryptoOptions {
  iv: CryptoJS.lib.WordArray;
  mode: typeof CryptoJS.mode.CBC;
  padding: typeof CryptoJS.pad.Pkcs7;
}

// 定义加密服务类
export class CryptoService {
  private readonly key: CryptoJS.lib.WordArray;
  private readonly iv: CryptoJS.lib.WordArray;

  constructor(config: CryptoConfig) {
    if (config.SECRET_KEY.length !== 32) {
      throw new Error("SECRET_KEY must be 32 characters long");
    }
    if (config.IV.length !== 16) {
      throw new Error("IV must be 16 characters long");
    }

    this.key = CryptoJS.enc.Utf8.parse(config.SECRET_KEY);
    this.iv = CryptoJS.enc.Utf8.parse(config.IV);
  }

  private get encryptOptions(): CryptoOptions {
    return {
      iv: this.iv,
      mode: CryptoJS.mode.CBC,
      padding: CryptoJS.pad.Pkcs7
    };
  }

  /**
   * 加密文本
   * @param text 要加密的文本
   * @returns 加密后的字符串
   * @throws Error 如果加密失败
   */
  public encrypt(text: string): string {
    try {
      if (!text) {
        throw new Error("Text to encrypt cannot be empty");
      }

      const encrypted = CryptoJS.AES.encrypt(
        text,
        this.key,
        this.encryptOptions
      );

      return encrypted.toString();
    } catch (error) {
      throw new Error(`Encryption failed: ${(error as Error).message}`);
    }
  }

  /**
   * 解密文本
   * @param encryptedText 加密的文本
   * @returns 解密后的原文
   * @throws Error 如果解密失败
   */
  public decrypt(encryptedText: string): string {
    try {
      if (!encryptedText) {
        throw new Error("Text to decrypt cannot be empty");
      }

      const decrypted = CryptoJS.AES.decrypt(
        encryptedText,
        this.key,
        this.encryptOptions
      );

      const result = decrypted.toString(CryptoJS.enc.Utf8);

      if (!result) {
        throw new Error("Decryption resulted in empty string");
      }

      return result;
    } catch (error) {
      throw new Error(`Decryption failed: ${(error as Error).message}`);
    }
  }
}
// 创建单例实例
const cryptoConfig: CryptoConfig = {
  SECRET_KEY: "mK9bP2vN8xL5tR7hJ4fD1cA3gE6iQ0wS", // 32字符密钥
  IV: "uY5nM2kX7pJ9vB4c" // 16字符IV
};

export const cryptoService = new CryptoService(cryptoConfig);
