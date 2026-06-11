export type ServiceWorkerRegisterFn = (scriptURL: URL, options?: RegistrationOptions) => Promise<unknown>;

export async function registerAppServiceWorker(
  registerFn: ServiceWorkerRegisterFn | undefined,
  scriptURL: URL,
  logger: (message: string, error?: unknown) => void = console.warn,
) {
  if (!registerFn) return false;
  try {
    await registerFn(scriptURL, { type: "module" });
    return true;
  } catch (error) {
    logger("Falha ao registrar o service worker", error);
    return false;
  }
}
