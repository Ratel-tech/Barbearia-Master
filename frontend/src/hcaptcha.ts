type HcaptchaRenderOptions = {
  sitekey: string;
  theme?: "light" | "dark";
  size?: "normal" | "compact";
  callback?: (token: string) => void;
  "error-callback"?: () => void;
  "expired-callback"?: () => void;
  tabindex?: number;
};

export interface HcaptchaApi {
  render(container: HTMLElement | string, options: HcaptchaRenderOptions): number;
  reset(widgetId?: number): void;
  remove(widgetId?: number): void;
}

declare global {
  interface Window {
    hcaptcha?: HcaptchaApi;
  }
}

const scriptId = "stitch-hcaptcha-script";
let scriptPromise: Promise<HcaptchaApi> | null = null;

export function loadHcaptcha(): Promise<HcaptchaApi> {
  if (window.hcaptcha) {
    return Promise.resolve(window.hcaptcha);
  }
  if (scriptPromise) {
    return scriptPromise;
  }

  scriptPromise = new Promise<HcaptchaApi>((resolve, reject) => {
    const existing = document.getElementById(scriptId) as HTMLScriptElement | null;
    if (existing && window.hcaptcha) {
      resolve(window.hcaptcha);
      return;
    }

    const script = existing ?? document.createElement("script");
    script.id = scriptId;
    script.src = "https://js.hcaptcha.com/1/api.js?render=explicit&hl=pt-BR";
    script.async = true;
    script.defer = true;
    script.onload = () => {
      if (window.hcaptcha) {
        resolve(window.hcaptcha);
        return;
      }
      reject(new Error("hCaptcha indisponivel"));
    };
    script.onerror = () => reject(new Error("nao foi possivel carregar o hCaptcha"));

    if (!existing) {
      document.head.appendChild(script);
    }
  });

  return scriptPromise;
}
