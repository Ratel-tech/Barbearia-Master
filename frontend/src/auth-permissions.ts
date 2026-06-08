export type Role = "owner" | "admin" | "barber" | "reception";
export type AppPage = "agenda" | "clientes" | "servicos" | "financeiro" | "profissionais" | "comissoes";

export function pagesForRole(role: Role): AppPage[] {
  if (role === "barber") return ["agenda", "comissoes"];
  return ["agenda", "clientes", "servicos", "financeiro", "profissionais"];
}

export function canAccessPage(role: Role, page: AppPage) {
  return pagesForRole(role).includes(page);
}
