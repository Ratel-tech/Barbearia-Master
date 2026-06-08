export type CatalogService = {
  active: boolean;
};

export function appointmentServices<T extends CatalogService>(services: T[]) {
  return services.filter((service) => service.active);
}

export function serviceStatusLabel(active: boolean) {
  return active ? "Ativo" : "Inativo";
}
