export type InstallAction = "native" | "ios-help";
export type InstallPlatform = "android" | "ios" | "desktop";

export type InstallPromptInput = {
  dismissed: boolean;
  hasNativePrompt: boolean;
  isStandalone: boolean;
  mobile: boolean;
  platform: InstallPlatform;
};

export function installPromptState(input: InstallPromptInput): { action: InstallAction | null; visible: boolean } {
  if (input.dismissed || input.isStandalone || !input.mobile) {
    return { action: null, visible: false };
  }

  if (input.hasNativePrompt) {
    return { action: "native", visible: true };
  }

  if (input.platform === "ios") {
    return { action: "ios-help", visible: true };
  }

  return { action: null, visible: false };
}

export function installText(action: InstallAction) {
  if (action === "ios-help") {
    return {
      button: "Ver instrução",
      message: "No iPhone, adicione pela opção Compartilhar.",
      title: "Instalar no celular",
    };
  }

  return {
    button: "Instalar app",
    message: "Abra o sistema direto pela tela inicial do celular.",
    title: "Barbearia Mestre",
  };
}

export function detectInstallPlatform(userAgent: string): InstallPlatform {
  const normalized = userAgent.toLowerCase();
  if (/iphone|ipad|ipod/.test(normalized)) return "ios";
  if (/android|mobile/.test(normalized)) return "android";
  return "desktop";
}
