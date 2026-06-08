import { Fragment, useCallback, useEffect, useState } from "react";
import type { FormEvent, ReactNode } from "react";
import {
  Badge,
  Banknote,
  CalendarDays,
  CheckCircle2,
  Clock3,
  DollarSign,
  Edit3,
  Menu,
  Plus,
  Scissors,
  Trash2,
  TrendingUp,
  UserPlus,
  Users,
  WalletCards,
  X,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { api } from "./api";
import type { Appointment, AuthUser, Barber, Client, Commission, ExtraExpense, Overview, Service } from "./api";
import { appointmentStatusClass, canCheckoutAppointment, canEditAppointment } from "./appointment-status";
import { accountLabel, loginPayload } from "./auth-account";
import type { AccountType } from "./auth-account";
import { validateBarberRequiredFields } from "./barber-validation";
import { pagesForRole } from "./auth-permissions";
import type { AppPage } from "./auth-permissions";
import { businessHours } from "./calendar-hours";
import { amountToCents, checkoutPaymentSummary } from "./checkout-payments";
import type { CheckoutPaymentDraft } from "./checkout-payments";
import { clientDraftFromSearch, clientSearchMatches } from "./client-search";
import { formatMobilePhoneInput, validateClientRequiredFields } from "./client-validation";
import { addMonths, buildMonthDays, monthLabel } from "./date-navigation";
import { passwordResetPayload } from "./password-reset";
import { commissionSummary, professionalAgendaSummary } from "./professional-portal";
import { appointmentServices, serviceStatusLabel } from "./service-catalog";

type Page = AppPage;
type AppointmentDraft = {
  barberId?: number;
  startsAt?: string;
};
type IconName =
  | "agenda"
  | "clientes"
  | "servicos"
  | "financeiro"
  | "profissionais"
  | "adicionar"
  | "editar"
  | "excluir"
  | "fechar"
  | "atendimentos"
  | "relogio"
  | "dinheiro"
  | "grafico"
  | "carteira"
  | "sucesso"
  | "novoCliente"
  | "menu";

const icons: Record<IconName, LucideIcon> = {
  agenda: CalendarDays,
  clientes: Users,
  servicos: Scissors,
  financeiro: Banknote,
  profissionais: Badge,
  adicionar: Plus,
  editar: Edit3,
  excluir: Trash2,
  fechar: X,
  atendimentos: CheckCircle2,
  relogio: Clock3,
  dinheiro: DollarSign,
  grafico: TrendingUp,
  carteira: WalletCards,
  sucesso: CheckCircle2,
  novoCliente: UserPlus,
  menu: Menu,
};

const money = (cents = 0) =>
  new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(cents / 100);

function Icon({ name, className }: { name: IconName; className?: string }) {
  const Component = icons[name];
  return <Component aria-hidden="true" className={className} size={18} strokeWidth={1.8} />;
}

function Metric({ label, value, icon, tone }: { label: string; value: string; icon: IconName; tone?: string }) {
  return (
    <article className="card metric">
      <span className="eyebrow">{label}</span>
      <strong className={tone}>{value}</strong>
      <Icon name={icon} className="ghost" />
    </article>
  );
}

export default function App() {
  const [page, setPage] = useState<Page>("agenda");
  const [sidebarExpanded, setSidebarExpanded] = useState(false);
  const [auth, setAuth] = useState<AuthUser | null>(null);
  const [authChecked, setAuthChecked] = useState(false);
  const [query, setQuery] = useState("");
  const [clients, setClients] = useState<Client[]>([]);
  const [barbers, setBarbers] = useState<Barber[]>([]);
  const [services, setServices] = useState<Service[]>([]);
  const [appointments, setAppointments] = useState<Appointment[]>([]);
  const [extraExpenses, setExtraExpenses] = useState<ExtraExpense[]>([]);
  const [overview, setOverview] = useState<Overview | null>(null);
  const [modal, setModal] = useState<"appointment" | "client" | "service" | "barber" | "checkout" | "commissions" | null>(null);
  const [appointmentDraft, setAppointmentDraft] = useState<AppointmentDraft>({});
  const [selectedAppointment, setSelectedAppointment] = useState<Appointment | null>(null);
  const [editingAppointment, setEditingAppointment] = useState<Appointment | null>(null);
  const [selectedBarber, setSelectedBarber] = useState<Barber | null>(null);
  const [editingBarber, setEditingBarber] = useState<Barber | null>(null);
  const [editingClient, setEditingClient] = useState<Client | null>(null);
  const [editingService, setEditingService] = useState<Service | null>(null);
  const [toast, setToast] = useState("");
  const [loading, setLoading] = useState(true);

  const load = useCallback(async (currentAuth: AuthUser) => {
    setLoading(true);
    if (currentAuth.role === "barber") {
      const nextAppointments = await api.appointments();
      setOverview(null);
      setClients([]);
      setServices([]);
      setExtraExpenses([]);
      setAppointments(nextAppointments);
      setBarbers([
        {
          id: currentAuth.barber_id ?? currentAuth.id,
          name: currentAuth.name,
          document: "",
          email: currentAuth.email,
          specialty: "",
          status: "active",
          monthly_commission_cents: 0,
          monthly_tips_cents: 0,
          completed_services: 0,
        },
      ]);
    } else {
      const [nextOverview, nextClients, nextBarbers, nextServices, nextAppointments, nextExtraExpenses] = await Promise.all([
        api.overview(),
        api.clients(),
        api.barbers(),
        api.services(),
        api.appointments(),
        api.extraExpenses(),
      ]);
      setOverview(nextOverview);
      setClients(nextClients);
      setBarbers(nextBarbers);
      setServices(nextServices);
      setAppointments(nextAppointments);
      setExtraExpenses(nextExtraExpenses);
    }
    setLoading(false);
  }, []);

  async function reload() {
    if (!auth) return;
    await load(auth);
  }

  useEffect(() => {
    let active = true;
    async function bootstrap() {
      try {
        if (!api.hasToken()) {
          if (active) {
            setAuthChecked(true);
            setLoading(false);
          }
          return;
        }
        const nextAuth = await api.me();
        if (!active) return;
        setAuth(nextAuth);
        setPage((currentPage) => (pagesForRole(nextAuth.role).includes(currentPage) ? currentPage : "agenda"));
        await load(nextAuth);
      } catch (error) {
        api.clearToken();
        if (active) setToast(`Sessão indisponível: ${(error as Error).message}`);
      } finally {
        if (active) {
          setAuthChecked(true);
          setLoading(false);
        }
      }
    }
    void bootstrap();
    return () => {
      active = false;
    };
  }, [load]);

  function notify(message: string) {
    setToast(message);
    window.setTimeout(() => setToast(""), 3500);
  }

  async function enter(response: { token: string; user: AuthUser }) {
    api.setToken(response.token);
    setAuth(response.user);
    setPage("agenda");
    await load(response.user);
  }

  function logout() {
    api.clearToken();
    setAuth(null);
    setOverview(null);
    setClients([]);
    setBarbers([]);
    setServices([]);
    setAppointments([]);
    setExtraExpenses([]);
    setPage("agenda");
  }

  const filteredClients = clients.filter((client) => client.name.toLowerCase().includes(query.toLowerCase()));
  const filteredServices = services.filter((service) => service.name.toLowerCase().includes(query.toLowerCase()));
  const filteredBarbers = barbers.filter((barber) => barber.name.toLowerCase().includes(query.toLowerCase()));

  const allNav: Array<{ id: Page; label: string; icon: IconName }> = [
    { id: "agenda", label: "Calendário / Agenda", icon: "agenda" },
    { id: "clientes", label: "Clientes", icon: "clientes" },
    { id: "servicos", label: "Serviços", icon: "servicos" },
    { id: "financeiro", label: "Financeiro", icon: "financeiro" },
    { id: "profissionais", label: "Profissionais", icon: "profissionais" },
    { id: "comissoes", label: "Minhas Comissões", icon: "carteira" },
  ];
  const nav = allNav.filter((item) => auth && pagesForRole(auth.role).includes(item.id));
  const isBarber = auth?.role === "barber";

  if (!authChecked || (loading && !auth)) {
    return <div className="auth-loading">Carregando...</div>;
  }

  if (!auth) {
    return <AuthScreen onEnter={enter} />;
  }

  if (isBarber) {
    return (
      <ProfessionalShell
        appointments={appointments}
        auth={auth}
        loading={loading}
        onLogout={logout}
        page={page}
        setPage={setPage}
      />
    );
  }

  return (
    <div className={`app-shell ${sidebarExpanded ? "is-sidebar-expanded" : ""}`}>
      <header className="app-topnav">
        <div className="brand-top">
          <h1>Barbearia Mestre</h1>
          <p>{auth.barbershop_name}</p>
        </div>
        <input className="search" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Buscar no sistema..." />
        <div className="session-menu">
          <span>{auth.name}</span>
          <button className="btn ghost compact" onClick={logout}>Sair</button>
        </div>
      </header>

      <aside className={`sidebar ${sidebarExpanded ? "expanded" : ""}`}>
        <button
          aria-label={sidebarExpanded ? "Recolher menu" : "Expandir menu"}
          className="sidebar-toggle"
          onClick={() => setSidebarExpanded((expanded) => !expanded)}
          title={sidebarExpanded ? "Recolher menu" : "Expandir menu"}
        >
          <Icon name="menu" />
          <span>{sidebarExpanded ? "Recolher" : "Menu"}</span>
        </button>
        <nav className="nav">
          {nav.map((item) => (
            <button
              key={item.id}
              aria-label={item.label}
              className={page === item.id ? "active" : ""}
              onClick={() => setPage(item.id)}
              title={item.label}
            >
              <Icon name={item.icon} />
              <span>{item.label}</span>
            </button>
          ))}
        </nav>
      </aside>

      <main className="main">
        <div className="main-content">
          <section className="contextbar" aria-label="Contexto da tela atual">
            <div>
              <p className="eyebrow">Gestão Profissional</p>
              <h2 className="page-title">{titleFor(page)}</h2>
            </div>
            <div className="context-actions" aria-hidden="true">
              <span>Atendimento</span>
              <span>Agenda</span>
              <span>Gestão</span>
            </div>
          </section>

          {loading ? (
            <div className="grid cols-3"><div className="skeleton" /><div className="skeleton" /><div className="skeleton" /></div>
          ) : (
            <>
              {page === "agenda" && (
                <Agenda
                  appointments={appointments}
                  barbers={barbers}
                  canManage={!isBarber}
                  onNew={(draft = {}) => {
                    setEditingAppointment(null);
                    setAppointmentDraft(draft);
                    setModal("appointment");
                  }}
                  onSlotClick={(draft) => {
                    setEditingAppointment(null);
                    setAppointmentDraft(draft);
                    setModal("appointment");
                  }}
                  onCheckout={(appointment) => {
                    setSelectedAppointment(appointment);
                    setModal("checkout");
                  }}
                  onManage={(appointment) => {
                    setEditingAppointment(appointment);
                    setAppointmentDraft({});
                    setModal("appointment");
                  }}
                />
              )}
              {page === "clientes" && (
                <Clients
                  clients={filteredClients}
                  onNew={() => {
                    setEditingClient(null);
                    setModal("client");
                  }}
                  onEdit={(client) => {
                    setEditingClient(client);
                    setModal("client");
                  }}
                />
              )}
              {page === "servicos" && (
                <Services
                  services={filteredServices}
                  onNew={() => {
                    setEditingService(null);
                    setModal("service");
                  }}
                  onEdit={(service) => {
                    setEditingService(service);
                    setModal("service");
                  }}
                />
              )}
              {page === "comissoes" && auth.barber_id && <ProfessionalCommissions barberId={auth.barber_id} />}
              {page === "financeiro" && (
                <Finance
                  overview={overview}
                  appointments={appointments}
                  expenses={extraExpenses}
                  onSaved={() => reload().then(() => notify("Gasto registrado"))}
                />
              )}
              {page === "profissionais" && (
                <Barbers
                  barbers={filteredBarbers}
                  onNew={() => {
                    setEditingBarber(null);
                    setModal("barber");
                  }}
                  onEdit={(barber) => {
                    setEditingBarber(barber);
                    setModal("barber");
                  }}
                  onDelete={async (barber) => {
                    if (!window.confirm(`Excluir ${barber.name}? O histórico de atendimentos será preservado.`)) return;
                    await api.deleteBarber(barber.id);
                    await reload();
                    notify("Profissional excluído");
                  }}
                  onCommissions={(barber) => {
                    setSelectedBarber(barber);
                    setModal("commissions");
                  }}
                />
              )}
            </>
          )}
        </div>
      </main>

      {modal === "appointment" && (
        <AppointmentModal
          clients={clients}
          barbers={barbers}
          services={services}
          draft={appointmentDraft}
          appointment={editingAppointment}
          onClose={() => setModal(null)}
          onSaved={() => reload().then(() => notify(editingAppointment ? "Agendamento atualizado" : "Agendamento salvo"))}
        />
      )}
      {modal === "client" && (
        <ClientModal
          client={editingClient}
          onClose={() => setModal(null)}
          onSaved={() => reload().then(() => notify(editingClient ? "Cliente atualizado" : "Cliente cadastrado"))}
        />
      )}
      {modal === "service" && (
        <ServiceModal
          service={editingService}
          onClose={() => setModal(null)}
          onSaved={() => reload().then(() => notify(editingService ? "Serviço atualizado" : "Serviço cadastrado"))}
        />
      )}
      {modal === "barber" && (
        <BarberModal
          barber={editingBarber}
          onClose={() => setModal(null)}
          onSaved={() => reload().then(() => notify(editingBarber ? "Profissional atualizado" : "Profissional cadastrado"))}
        />
      )}
      {modal === "checkout" && selectedAppointment && (
        <CheckoutModal appointment={selectedAppointment} onClose={() => setModal(null)} onSaved={() => reload().then(() => notify("Comanda fechada"))} />
      )}
      {modal === "commissions" && selectedBarber && (
        <CommissionsModal barber={selectedBarber} onClose={() => setModal(null)} onSaved={() => notify("Comissao atualizada")} />
      )}
      {toast && <div className="toast">{toast}</div>}
    </div>
  );
}

function ProfessionalShell({ auth, appointments, loading, page, setPage, onLogout }: {
  auth: AuthUser;
  appointments: Appointment[];
  loading: boolean;
  page: Page;
  setPage: (page: Page) => void;
  onLogout: () => void;
}) {
  const activePage = page === "comissoes" ? "comissoes" : "agenda";
  return (
    <div className="professional-shell">
      <header className="professional-topbar">
        <div>
          <p className="eyebrow">Portal do Profissional</p>
          <h1>{auth.name}</h1>
          <span>{auth.barbershop_name}</span>
        </div>
        <button className="btn ghost compact" onClick={onLogout}>Sair</button>
      </header>

      <main className="professional-main">
        {loading ? (
          <div className="grid"><div className="skeleton" /><div className="skeleton" /></div>
        ) : (
          <>
            {activePage === "agenda" && <ProfessionalAgenda appointments={appointments} />}
            {activePage === "comissoes" && auth.barber_id && <ProfessionalCommissions barberId={auth.barber_id} professional />}
          </>
        )}
      </main>

      <nav className="professional-bottom-nav" aria-label="Navegação do profissional">
        <button className={activePage === "agenda" ? "active" : ""} onClick={() => setPage("agenda")}>
          <Icon name="agenda" />
          <span>Agenda</span>
        </button>
        <button className={activePage === "comissoes" ? "active" : ""} onClick={() => setPage("comissoes")}>
          <Icon name="carteira" />
          <span>Comissões</span>
        </button>
      </nav>
    </div>
  );
}

function ProfessionalAgenda({ appointments }: { appointments: Appointment[] }) {
  const summary = professionalAgendaSummary(appointments, localDate());
  return (
    <section className="grid professional-stack">
      <div className="professional-hero card">
        <p className="eyebrow">Agenda de Hoje</p>
        <h2>{summary.nextAppointment ? summary.nextAppointment.client_name : "Sem próximo atendimento"}</h2>
        <p>{summary.nextAppointment ? `${formatTime(summary.nextAppointment.starts_at)} - ${summary.nextAppointment.services}` : "Acompanhe seus próximos horários por aqui."}</p>
      </div>
      <div className="grid cols-3">
        <Metric label="Hoje" value={`${summary.today.length}`} icon="agenda" />
        <Metric label="Abertos" value={`${summary.openCount}`} icon="relogio" />
        <Metric label="Concluídos" value={`${summary.completedCount}`} icon="sucesso" />
      </div>
      <section className="professional-list">
        {summary.today.length === 0 ? (
          <EmptyState title="Nenhum atendimento hoje" body="Quando o salão agendar clientes para você, eles aparecem nesta lista." />
        ) : (
          summary.today.map((appointment) => (
            <article className="professional-appointment-card" key={appointment.id}>
              <div>
                <strong>{formatTime(appointment.starts_at)}</strong>
                <h3>{appointment.client_name}</h3>
                <p>{appointment.services}</p>
              </div>
              <span className={`appointment-status ${appointmentStatusClass(appointment.status)}`}>{statusLabel(appointment.status)}</span>
            </article>
          ))
        )}
      </section>
    </section>
  );
}

function formatTime(value: string) {
  return value.slice(11, 16);
}

function AuthScreen({ onEnter }: { onEnter: (response: { token: string; user: AuthUser }) => Promise<void> }) {
  const [mode, setMode] = useState<"login" | "register" | "forgot" | "reset">("login");
  const [accountType, setAccountType] = useState<AccountType>("establishment");
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");
  const [busy, setBusy] = useState(false);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setError("");
    setSuccess("");
    const data = Object.fromEntries(new FormData(event.currentTarget));
    try {
      if (mode === "forgot") {
        const response = await api.forgotPassword(passwordResetPayload(String(data.email), accountType));
        setSuccess(response.reset_token ? `${response.message}. Codigo: ${response.reset_token}` : response.message);
        return;
      }
      if (mode === "reset") {
        const response = await api.resetPassword(passwordResetPayload(String(data.token), String(data.password)));
        setSuccess(response.message);
        setMode("login");
        return;
      }
      const response = mode === "login"
        ? await api.login(loginPayload(String(data.email), String(data.password), accountType))
        : await api.registerBarbershop({
          barbershop_name: String(data.barbershop_name),
          owner_name: String(data.owner_name),
          email: String(data.email),
          password: String(data.password),
        });
      await onEnter(response);
    } catch (caught) {
      setError((caught as Error).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="auth-page">
      <section className="auth-panel">
        <div className="brand-top">
          <h1>Barbearia Mestre</h1>
          <p>Gestão de Elite</p>
        </div>
        <div className="segmented">
          <button className={mode === "login" ? "active" : ""} onClick={() => setMode("login")}>Entrar</button>
          <button className={mode === "register" ? "active" : ""} onClick={() => setMode("register")}>Cadastrar barbearia</button>
        </div>
        <form className="form-grid auth-form" onSubmit={submit}>
          {(mode === "login" || mode === "forgot") && (
            <div className="segmented full compact-segmented" aria-label="Tipo de acesso">
              {(["establishment", "professional"] as const).map((type) => (
                <button
                  className={accountType === type ? "active" : ""}
                  key={type}
                  onClick={() => setAccountType(type)}
                  type="button"
                >
                  {accountLabel(type)}
                </button>
              ))}
            </div>
          )}
          {mode === "register" && (
            <>
              <Field name="barbershop_name" label="Nome da barbearia" required />
              <Field name="owner_name" label="Nome do responsável" required />
            </>
          )}
          {mode !== "reset" && (
            <Field name="email" label="E-mail" type="email" required />
          )}
          {mode === "reset" && <Field name="token" label="Código de recuperação" required />}
          {mode !== "forgot" && (
            <Field name="password" label={mode === "reset" ? "Nova senha" : "Senha"} type="password" required />
          )}
          {error && <p className="form-error full">{error}</p>}
          {success && <p className="form-success full">{success}</p>}
          <button className="btn primary full" disabled={busy}>{busy ? "Validando..." : authSubmitLabel(mode)}</button>
          <div className="auth-links full">
            {mode === "login" && <button type="button" onClick={() => setMode("forgot")}>Esqueci minha senha</button>}
            {mode === "forgot" && <button type="button" onClick={() => setMode("reset")}>Já tenho código</button>}
            {(mode === "forgot" || mode === "reset") && <button type="button" onClick={() => setMode("login")}>Voltar para entrar</button>}
          </div>
        </form>
      </section>
    </main>
  );
}

function authSubmitLabel(mode: "login" | "register" | "forgot" | "reset") {
  return {
    login: "Entrar no sistema",
    register: "Criar barbearia",
    forgot: "Enviar código",
    reset: "Atualizar senha",
  }[mode];
}

function titleFor(page: Page) {
  return { agenda: "Agenda", clientes: "Clientes", servicos: "Catálogo de Serviços", financeiro: "Financeiro", profissionais: "Profissionais", comissoes: "Minhas Comissões" }[page];
}

function statusLabel(status: string) {
  return {
    scheduled: "agendado",
    in_chair: "em atendimento",
    completed: "concluído",
    cancelled: "cancelado",
    active: "ativo",
    inactive: "inativo",
  }[status] ?? status;
}

function EmptyState({ title, body, action }: { title: string; body: string; action?: ReactNode }) {
  return (
    <article className="card empty-state">
      <div>
        <p className="eyebrow">Sistema limpo</p>
        <h3>{title}</h3>
        <p>{body}</p>
      </div>
      {action}
    </article>
  );
}

function Agenda({ appointments, barbers, canManage, onNew, onSlotClick, onCheckout, onManage }: {
  appointments: Appointment[];
  barbers: Barber[];
  canManage?: boolean;
  onNew: (draft?: AppointmentDraft) => void;
  onSlotClick: (draft: AppointmentDraft) => void;
  onCheckout: (appointment: Appointment) => void;
  onManage: (appointment: Appointment) => void;
}) {
  const today = localDate();
  const [selectedDate, setSelectedDate] = useState(today);
  const hours = businessHours();
  const monthDays = buildMonthDays(selectedDate, today);
  const visibleBarbers = barbers;
  const selectedAppointments = appointments.filter((appointment) => appointment.starts_at.slice(0, 10) === selectedDate);
  const openSelectedAppointments = selectedAppointments.filter((appointment) => appointment.status !== "completed").length;
  const inProgressSelectedAppointments = selectedAppointments.filter((appointment) => appointment.status === "in_chair").length;
  return (
    <section className="grid">
      <div className="toolbar">
        <div className="grid cols-3" style={{ flex: 1 }}>
          <Metric label="Total do Dia" value={`${selectedAppointments.length}`} icon="atendimentos" />
          <Metric label="Abertos no Dia" value={`${openSelectedAppointments}`} icon="relogio" />
          <Metric label="Em Andamento" value={`${inProgressSelectedAppointments}`} icon="dinheiro" />
        </div>
        {canManage !== false && <button className="btn primary" onClick={() => onNew({ startsAt: `${selectedDate}T09:00` })}><Icon name="adicionar" /> Novo Agendamento</button>}
      </div>
      {visibleBarbers.length === 0 ? (
        <EmptyState
          title="Agenda vazia"
          body="Cadastre profissionais, serviços e clientes para começar a montar a agenda."
          action={canManage !== false ? <button className="btn primary" onClick={() => onNew({ startsAt: `${selectedDate}T09:00` })}><Icon name="adicionar" /> Criar agendamento</button> : undefined}
        />
      ) : (
      <div className="agenda-workspace">
        <aside className="mini-calendar card">
          <div className="mini-calendar-header">
            <button className="icon-button" onClick={() => setSelectedDate(addMonths(selectedDate, -1))} aria-label="Mês anterior">‹</button>
            <strong>{monthLabel(selectedDate)}</strong>
            <button className="icon-button" onClick={() => setSelectedDate(addMonths(selectedDate, 1))} aria-label="Próximo mês">›</button>
          </div>
          <div className="weekdays">
            {["dom", "seg", "ter", "qua", "qui", "sex", "sáb"].map((day) => <span key={day}>{day}</span>)}
          </div>
          <div className="month-grid">
            {monthDays.map((day) => (
              <button
                className={`month-day ${day.currentMonth ? "" : "muted"} ${day.today ? "today" : ""} ${day.date === selectedDate ? "selected" : ""}`}
                key={day.date}
                onClick={() => setSelectedDate(day.date)}
              >
                {day.day}
              </button>
            ))}
          </div>
        </aside>
        <div className="calendar" style={{ gridTemplateColumns: `90px repeat(${visibleBarbers.length}, minmax(180px, 1fr))` }}>
          <div className="head">Hora</div>
          {visibleBarbers.map((barber) => <div className="head" key={barber.id}>{barber.name}</div>)}
          {hours.map((hour) => (
            <Fragment key={hour}>
              <div className="slot-time" key={`${hour}-time`}>{hour}</div>
              {visibleBarbers.map((barber) => {
                const item = selectedAppointments.find((appointment) => appointment.barber_id === barber.id && appointment.starts_at.includes(`T${hour}`));
                const canCheckout = item ? canManage !== false && canCheckoutAppointment(item.status) : false;
                const canManageAppointment = item ? canManage !== false && canEditAppointment(item.status) : false;
                return (
                  <div className="calendar-cell" key={`${hour}-${barber.id}`}>
                    {item && (
                      <div className={`appointment ${appointmentStatusClass(item.status)}`}>
                        <strong>{item.client_name}</strong>
                        <span>{item.services}</span>
                        <span>{money(item.total_cents)} - {statusLabel(item.status)}</span>
                        {canManageAppointment && (
                          <button type="button" className="btn ghost compact" onClick={() => onManage(item)}>
                            <Icon name="editar" /> Editar
                          </button>
                        )}
                        {canCheckout && (
                          <button type="button" className="btn primary compact" onClick={() => onCheckout(item)}>
                            Fechar
                          </button>
                        )}
                      </div>
                    )}
                    {!item && canManage !== false && (
                      <button
                        aria-label={`Agendar ${barber.name} as ${hour}`}
                        className="calendar-slot-button"
                        onClick={() => onSlotClick({ barberId: barber.id, startsAt: `${selectedDate}T${hour}` })}
                      />
                    )}
                  </div>
                );
              })}
            </Fragment>
          ))}
        </div>
      </div>
      )}
    </section>
  );
}

function localDate() {
  const now = new Date();
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function Clients({ clients, onNew, onEdit }: { clients: Client[]; onNew: () => void; onEdit: (client: Client) => void }) {
  return (
    <section className="grid">
      <div className="toolbar">
        <Metric label="Total de Clientes" value={`${clients.length}`} icon="clientes" />
        <button className="btn primary" onClick={onNew}><Icon name="novoCliente" /> Novo Cliente</button>
      </div>
      {clients.length === 0 ? (
        <EmptyState title="Nenhum cliente cadastrado" body="A base começa vazia. Cadastre o primeiro cliente para usar em agendamentos." />
      ) : (
        <div className="list">
          {clients.map((client) => (
          <article className="list-item" key={client.id}>
            <div>
              <h3>{client.name}</h3>
              <p>{client.phone} - {client.email ?? "sem email"} - {client.haircut_frequency ?? "frequencia nao definida"}</p>
            </div>
            <div className="row" style={{ alignItems: "center", gap: "1.5rem" }}>
              <div className="price">{money(client.total_spent_cents)}</div>
              <button className="btn ghost compact" onClick={() => onEdit(client)}><Icon name="editar" /> Editar</button>
            </div>
          </article>
          ))}
        </div>
      )}
    </section>
  );
}

function Services({ services, onNew, onEdit }: { services: Service[]; onNew: () => void; onEdit: (service: Service) => void }) {
  const featured = services.find((service) => service.active) ?? services[0];
  const remainingServices = services.filter((service) => service.id !== featured?.id);
  return (
    <section className="grid">
      <div className="toolbar">
        <div>
          <p className="eyebrow">Catálogo da barbearia</p>
          <h3>Serviços, preços e adicionais</h3>
        </div>
        <button className="btn primary" onClick={onNew}><Icon name="adicionar" /> Novo Serviço</button>
      </div>
      {services.length === 0 ? (
        <EmptyState title="Nenhum serviço cadastrado" body="Cadastre cortes, barba, combos e adicionais antes de criar agendamentos." />
      ) : (
      <>
      {featured && (
        <article className="card">
          <p className="eyebrow">Serviço destaque - {serviceStatusLabel(featured.active)}</p>
          <h3>{featured.name}</h3>
          <p>{featured.description}</p>
          <div className="row" style={{ justifyContent: "space-between" }}>
            <div className="price">{money(featured.price_cents)}</div>
            <button className="btn ghost compact" onClick={() => onEdit(featured)}><Icon name="editar" /> Editar</button>
          </div>
        </article>
      )}
      <div className="grid cols-3">
        {remainingServices.map((service) => (
          <article className="card subtle" key={service.id}>
            <p className="eyebrow">{service.category} - {service.duration_minutes} min - {serviceStatusLabel(service.active)}</p>
            <h3>{service.name}</h3>
            <p>{service.description}</p>
            <div className="row" style={{ justifyContent: "space-between" }}>
              <span className="price">{money(service.price_cents)}</span>
              <button className="btn ghost compact" onClick={() => onEdit(service)}><Icon name="editar" /> Editar</button>
            </div>
          </article>
        ))}
      </div>
      </>
      )}
    </section>
  );
}

function Finance({ overview, appointments, expenses, onSaved }: {
  overview: Overview | null;
  appointments: Appointment[];
  expenses: ExtraExpense[];
  onSaved: () => void;
}) {
  async function submitExpense(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = Object.fromEntries(new FormData(event.currentTarget));
    await api.createExtraExpense({
      description: String(data.description),
      amount_cents: Math.round(Number(data.amount) * 100),
    });
    event.currentTarget.reset();
    onSaved();
  }

  return (
    <section className="grid">
      <div className="grid cols-4">
        <Metric label="Faturamento Total" value={money(overview?.revenue_cents)} icon="grafico" />
        <Metric label="Comissão Geral" value={money(overview?.commissions_cents)} icon="carteira" />
        <Metric label="Faturamento Líquido" value={money(overview?.net_revenue_cents)} icon="dinheiro" />
        <Metric label="Lucro Final" value={money(overview?.profit_cents)} icon="sucesso" tone={(overview?.profit_cents ?? 0) < 0 ? "danger" : ""} />
      </div>
      <div className="grid cols-2">
      <article className="card">
        <div className="toolbar">
          <div>
            <p className="eyebrow">Gastos da casa</p>
            <h3>Gastos extras</h3>
          </div>
          <span className="price">{money(overview?.extra_expenses_cents)}</span>
        </div>
        <form className="form-grid finance-expense-form" onSubmit={submitExpense}>
          <Field name="description" label="Tipo de gasto" placeholder="Papel, vassoura, aluguel..." />
          <Field name="amount" label="Valor" type="number" />
          <div className="row full"><button className="btn primary"><Icon name="adicionar" /> Registrar gasto</button></div>
        </form>
      </article>
      <article className="card">
        <h3>Últimos gastos</h3>
        {expenses.length === 0 ? (
          <EmptyState title="Nenhum gasto extra" body="Lance os gastos da casa para calcular o lucro final." />
        ) : (
        <table className="table">
          <tbody>
            {expenses.slice(0, 6).map((expense) => (
              <tr key={expense.id}>
                <td>{expense.description}</td>
                <td className="eyebrow">{new Date(expense.created_at).toLocaleDateString("pt-BR")}</td>
                <td style={{ textAlign: "right" }}>{money(expense.amount_cents)}</td>
              </tr>
            ))}
          </tbody>
        </table>
        )}
      </article>
      </div>
      <article className="card">
        <h3>Últimos atendimentos</h3>
        {appointments.length === 0 ? (
          <EmptyState title="Nenhuma comanda fechada" body="Os lançamentos aparecerão aqui depois dos primeiros atendimentos." />
        ) : (
        <table className="table">
          <tbody>
            {appointments.slice(0, 5).map((appointment) => (
              <tr key={appointment.id}>
                <td>{appointment.services}</td>
                <td>{appointment.barber_name}</td>
                <td>{statusLabel(appointment.status)}</td>
                <td style={{ textAlign: "right" }}>{money(appointment.total_cents)}</td>
              </tr>
            ))}
          </tbody>
        </table>
        )}
      </article>
    </section>
  );
}

function Barbers({ barbers, onNew, onEdit, onDelete, onCommissions }: {
  barbers: Barber[];
  onNew: () => void;
  onEdit: (barber: Barber) => void;
  onDelete: (barber: Barber) => void;
  onCommissions: (barber: Barber) => void;
}) {
  return (
    <section className="grid">
      <div className="toolbar">
        <Metric label="Profissionais ativos" value={`${barbers.filter((barber) => barber.status === "active").length}`} icon="profissionais" />
        <button className="btn primary" onClick={onNew}><Icon name="adicionar" /> Novo Profissional</button>
      </div>
      {barbers.length === 0 ? (
        <EmptyState title="Nenhum profissional cadastrado" body="Cadastre os barbeiros da equipe para liberar agenda e comissões." />
      ) : (
      <div className="grid cols-2">
        {barbers.map((barber) => (
          <article className="card" key={barber.id}>
            <div className="toolbar">
              <div>
                <p className="eyebrow"><span className="status-dot" /> {statusLabel(barber.status)}</p>
                <h3>{barber.name}</h3>
                <p>{barber.specialty}</p>
              </div>
              <div className="row barber-actions">
                <button className="btn ghost compact" onClick={() => onCommissions(barber)}><Icon name="carteira" /> Comissão</button>
                <button className="btn ghost compact" onClick={() => onEdit(barber)}><Icon name="editar" /> Editar</button>
                <button className="btn ghost compact danger-action" onClick={() => onDelete(barber)}><Icon name="excluir" /> Excluir</button>
              </div>
            </div>
            <div className="grid cols-3">
              <Metric label="Comissão Mês" value={money(barber.monthly_commission_cents)} icon="dinheiro" />
              <Metric label="Serviços" value={`${barber.completed_services}`} icon="servicos" />
              <Metric label="Gorjetas" value={money(barber.monthly_tips_cents)} icon="carteira" />
            </div>
          </article>
        ))}
      </div>
      )}
    </section>
  );
}

function ClientModal({ client, onClose, onSaved }: { client?: Client | null; onClose: () => void; onSaved: () => void }) {
  const [error, setError] = useState("");
  const isEditing = !!client;
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = Object.fromEntries(new FormData(event.currentTarget));
    const validationError = validateClientRequiredFields({
      name: String(data.name),
      phone: String(data.phone),
      document: String(data.document),
      haircut_frequency: String(data.haircut_frequency),
    });
    if (validationError) {
      setError(validationError);
      return;
    }
    try {
      const payload = {
        name: String(data.name),
        phone: String(data.phone),
        email: String(data.email),
        document: String(data.document),
        haircut_frequency: String(data.haircut_frequency),
      };
      if (isEditing) {
        await api.updateClient(client.id, payload);
      } else {
        await api.createClient(payload);
      }
      onSaved();
      onClose();
    } catch (caught) {
      setError((caught as Error).message);
    }
  }
  return (
    <Modal title={isEditing ? "Editar Cliente" : "Cadastro de Novo Cliente"} onClose={onClose}>
      <form className="form-grid" onSubmit={submit}>
        <Field name="name" label="Nome completo" defaultValue={client?.name} />
        <PhoneField name="phone" label="Celular com DDD" defaultValue={client?.phone} />
        <Field name="haircut_frequency" label="De quanto em quanto tempo corta o cabelo?" defaultValue={client?.haircut_frequency} />
        <Field name="document" label="CPF" required={false} defaultValue={client?.document} />
        <Field name="email" label="Email" type="email" required={false} defaultValue={client?.email} />
        {error && <p className="form-error full">{error}</p>}
        <div className="row full"><button className="btn primary">Salvar e continuar</button><button type="button" className="btn ghost" onClick={onClose}>Cancelar</button></div>
      </form>
    </Modal>
  );
}

function AppointmentModal({ clients, barbers, services, draft, appointment, onClose, onSaved }: {
  clients: Client[]; barbers: Barber[]; services: Service[]; draft: AppointmentDraft; appointment?: Appointment | null; onClose: () => void; onSaved: () => void;
}) {
  const isEditing = !!appointment;
  const [selectedServices, setSelectedServices] = useState<number[]>(
    appointment?.service_ids.split(",").map(Number).filter(Boolean) ?? [],
  );
  const [createdClients, setCreatedClients] = useState<Client[]>([]);
  const [selectedClientId, setSelectedClientId] = useState<number | null>(appointment?.client_id ?? null);
  const [clientSearch, setClientSearch] = useState(appointment?.client_name ?? "");
  const [clientFormOpen, setClientFormOpen] = useState(clients.length === 0);
  const [clientError, setClientError] = useState("");
  const [newClient, setNewClient] = useState({ name: "", phone: "", document: "", email: "", haircut_frequency: "" });
  const activeServices = appointmentServices(services);
  const hasPrerequisites = barbers.length > 0 && activeServices.length > 0;
  const createdClientIds = new Set(createdClients.map((client) => client.id));
  const availableClients = [...createdClients, ...clients.filter((client) => !createdClientIds.has(client.id))];
  const hasClientSearch = clientSearch.trim().length > 0;
  const matchingClients = hasClientSearch ? availableClients.filter((client) => clientSearchMatches(client, clientSearch)).slice(0, 6) : [];
  const shouldOfferClientCreation = hasClientSearch;

  function openClientForm() {
    const quickDraft = clientDraftFromSearch(clientSearch);
    setNewClient({
      name: /\d/.test(clientSearch) ? "" : clientSearch.trim(),
      phone: formatMobilePhoneInput(quickDraft.phone),
      document: quickDraft.document,
      email: "",
      haircut_frequency: "",
    });
    setClientFormOpen(true);
    setClientError("");
  }

  async function createClientFromAppointment() {
    const validationError = validateClientRequiredFields(newClient);
    if (validationError) {
      setClientError(validationError);
      return;
    }
    try {
      const created = await api.createClient({
        name: newClient.name,
        phone: newClient.phone,
        document: newClient.document,
        email: newClient.email,
        haircut_frequency: newClient.haircut_frequency,
      });
      setCreatedClients((current) => [created, ...current]);
      setSelectedClientId(created.id);
      setClientSearch(created.name);
      setClientFormOpen(false);
      setClientError("");
    } catch (caught) {
      setClientError((caught as Error).message);
    }
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = Object.fromEntries(new FormData(event.currentTarget));
    if (!selectedClientId) {
      setClientError("Selecione ou cadastre um cliente para continuar.");
      return;
    }
    if (selectedServices.length === 0) {
      setClientError("Selecione pelo menos um serviço para continuar.");
      return;
    }
    try {
      const payload = {
        client_id: selectedClientId,
        barber_id: Number(data.barber_id),
        service_ids: selectedServices,
        starts_at: String(data.starts_at),
        status: String(data.status || appointment?.status || "scheduled"),
      };
      if (isEditing) {
        await api.updateAppointment(appointment.id, payload);
      } else {
        await api.createAppointment(payload);
      }
      onSaved();
      onClose();
    } catch (caught) {
      setClientError((caught as Error).message);
    }
  }
  return (
    <Modal title={isEditing ? "Editar Agendamento" : "Novo Agendamento"} onClose={onClose} large>
      {!hasPrerequisites ? (
        <EmptyState
          title="Complete os cadastros antes"
          body="Para criar um agendamento, cadastre pelo menos um profissional e um serviço."
        />
      ) : (
      <form className="form-grid" onSubmit={submit}>
        <div className="field full client-picker">
          <div className="client-picker-header">
            <label className="label">Cliente</label>
            <button type="button" className="btn ghost compact" onClick={openClientForm}><Icon name="adicionar" /> Novo cliente</button>
          </div>
          <input
            className="client-search"
            value={clientSearch}
            onChange={(event) => {
              setClientSearch(event.target.value);
              setClientError("");
            }}
            placeholder="Buscar por nome, telefone ou CPF"
          />
          <input type="hidden" name="client_id" value={selectedClientId ?? ""} />
          {matchingClients.length > 0 && (
            <div className="client-results">
              {matchingClients.map((client) => (
                <button
                  type="button"
                  className={`client-result ${selectedClientId === client.id ? "selected" : ""}`}
                  key={client.id}
                  onClick={() => {
                    setSelectedClientId(client.id);
                    setClientSearch(client.name);
                    setClientError("");
                  }}
                >
                  <strong>{client.name}</strong>
                  <span>{client.phone}{client.document ? ` - CPF ${client.document}` : ""}</span>
                </button>
              ))}
            </div>
          )}
          {shouldOfferClientCreation && (
            <button type="button" className="client-create-option" onClick={openClientForm}>
              <Icon name="adicionar" /> Cadastrar novo cliente "{clientSearch.trim()}"
            </button>
          )}
          {clientFormOpen && (
            <div className="inline-client-form">
              <input
                value={newClient.name}
                onChange={(event) => setNewClient((current) => ({ ...current, name: event.target.value }))}
                placeholder="Nome completo"
              />
              <input
                value={newClient.phone}
                onChange={(event) => setNewClient((current) => ({ ...current, phone: formatMobilePhoneInput(event.target.value) }))}
                placeholder="Celular com DDD"
                maxLength={16}
              />
              <input
                value={newClient.haircut_frequency}
                onChange={(event) => setNewClient((current) => ({ ...current, haircut_frequency: event.target.value }))}
                placeholder="De quanto em quanto tempo corta?"
              />
              <input
                value={newClient.document}
                onChange={(event) => setNewClient((current) => ({ ...current, document: event.target.value }))}
                placeholder="CPF"
              />
              <input
                value={newClient.email}
                onChange={(event) => setNewClient((current) => ({ ...current, email: event.target.value }))}
                placeholder="Email"
                type="email"
              />
              <button type="button" className="btn primary" onClick={createClientFromAppointment}>
                <Icon name="adicionar" /> Salvar cliente
              </button>
              <button type="button" className="btn ghost" onClick={() => setClientFormOpen(false)}>Cancelar</button>
            </div>
          )}
          {clientError && <p className="form-error">{clientError}</p>}
        </div>
        <Select name="barber_id" label="Profissional" defaultValue={appointment?.barber_id ?? draft.barberId} options={barbers.map((barber) => [barber.id, barber.name])} />
        <Field name="starts_at" label="Data e hora" type="datetime-local" defaultValue={appointment?.starts_at ?? draft.startsAt} />
        {isEditing && (
          <Select
            name="status"
            label="Status"
            defaultValue={appointment.status}
            options={[["scheduled", "Agendado"], ["in_chair", "Em atendimento"], ["cancelled", "Cancelado"]]}
          />
        )}
        <div className="field full">
          <label className="label">Serviços</label>
          <div className="grid cols-3">
            {activeServices.map((service) => (
              <label className="card subtle" key={service.id}>
                <input
                  type="checkbox"
                  checked={selectedServices.includes(service.id)}
                  onChange={(event) => {
                    setSelectedServices((current) =>
                      event.target.checked ? [...current, service.id] : current.filter((id) => id !== service.id),
                    );
                  }}
                />
                <strong>{service.name}</strong>
                <p>{money(service.price_cents)}</p>
              </label>
            ))}
          </div>
        </div>
        <div className="row full"><button className="btn primary">Salvar Agendamento</button><button type="button" className="btn ghost" onClick={onClose}>Cancelar</button></div>
      </form>
      )}
    </Modal>
  );
}

function ServiceModal({ service, onClose, onSaved }: { service?: Service | null; onClose: () => void; onSaved: () => void }) {
  const isEditing = !!service;
  const [error, setError] = useState("");
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = Object.fromEntries(new FormData(event.currentTarget));
    const payload = {
      name: String(data.name),
      description: String(data.description),
      duration_minutes: Number(data.duration_minutes),
      price_cents: Math.round(Number(data.price) * 100),
      category: String(data.category),
      active: String(data.active || "true") === "true",
    };
    try {
      if (isEditing) {
        await api.updateService(service.id, payload);
      } else {
        await api.createService(payload);
      }
      onSaved();
      onClose();
    } catch (caught) {
      setError((caught as Error).message);
    }
  }
  return (
    <Modal title={isEditing ? "Editar Serviço" : "Novo Serviço"} onClose={onClose}>
      <form className="form-grid" onSubmit={submit}>
        <Field name="name" label="Nome" defaultValue={service?.name} />
        <Field name="price" label="Preço" type="number" defaultValue={service ? String(service.price_cents / 100) : undefined} />
        <Field name="duration_minutes" label="Duração em minutos" type="number" defaultValue={service ? String(service.duration_minutes) : undefined} />
        <Field name="category" label="Categoria" defaultValue={service?.category} required={false} />
        <Field name="description" label="Descrição" defaultValue={service?.description} required={false} full />
        {isEditing && (
          <Select
            name="active"
            label="Status"
            defaultValue={service.active ? "true" : "false"}
            options={[["true", "Ativo"], ["false", "Inativo"]]}
          />
        )}
        {error && <p className="form-error full">{error}</p>}
        <div className="row full"><button className="btn primary">Salvar Serviço</button><button type="button" className="btn ghost" onClick={onClose}>Cancelar</button></div>
      </form>
    </Modal>
  );
}

function BarberModal({ barber, onClose, onSaved }: { barber?: Barber | null; onClose: () => void; onSaved: () => void }) {
  const isEditing = !!barber;
  const [error, setError] = useState("");
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = Object.fromEntries(new FormData(event.currentTarget));
    const payload = {
      name: String(data.name),
      document: String(data.document),
      email: String(data.email),
      password: String(data.password),
      specialty: String(data.specialty),
      status: String(data.status || "active"),
    };
    const validationError = validateBarberRequiredFields({
      name: payload.name,
      email: payload.email,
      document: payload.document,
      password: payload.password,
      isEditing,
    });
    if (validationError) {
      setError(validationError);
      return;
    }
    try {
      if (isEditing) {
        await api.updateBarber(barber.id, {
          name: payload.name,
          document: payload.document,
          email: payload.email,
          password: payload.password.trim() ? payload.password : undefined,
          specialty: payload.specialty,
          status: payload.status,
        });
      } else {
        await api.createBarber(payload);
      }
      onSaved();
      onClose();
    } catch (caught) {
      setError((caught as Error).message);
    }
  }
  return (
    <Modal title={isEditing ? "Editar Profissional" : "Novo Profissional"} onClose={onClose}>
      <form className="form-grid" onSubmit={submit}>
        <Field name="name" label="Nome" defaultValue={barber?.name} />
        <Field name="document" label="CPF" defaultValue={barber?.document} />
        <Field name="email" label="Email" type="email" defaultValue={barber?.email} />
        <Field
          name="password"
          label={isEditing ? "Nova senha" : "Senha"}
          type="password"
          required={!isEditing}
          placeholder={isEditing ? "Deixe em branco para manter" : undefined}
        />
        <Field name="specialty" label="Especialidade" defaultValue={barber?.specialty} required={false} />
        {isEditing && (
          <Select
            name="status"
            label="Status"
            defaultValue={barber.status}
            options={[["active", "Ativo"], ["inactive", "Inativo"]]}
          />
        )}
        {error && <p className="form-error full">{error}</p>}
        <div className="row full"><button className="btn primary">Salvar Profissional</button><button type="button" className="btn ghost" onClick={onClose}>Cancelar</button></div>
      </form>
    </Modal>
  );
}

function CheckoutModal({ appointment, onClose, onSaved }: { appointment: Appointment; onClose: () => void; onSaved: () => void }) {
  const [payments, setPayments] = useState<CheckoutPaymentDraft[]>([
    { method: "cash", amount: (appointment.total_cents / 100).toFixed(2) },
  ]);
  const [error, setError] = useState("");
  const paymentOptions = [["cash", "Dinheiro"], ["pix", "Pix"], ["debit", "Debito"], ["credit", "Credito"]];
  const summary = checkoutPaymentSummary(appointment.total_cents, payments);

  function updatePayment(index: number, patch: Partial<CheckoutPaymentDraft>) {
    setPayments((current) => current.map((payment, paymentIndex) => (
      paymentIndex === index ? { ...payment, ...patch } : payment
    )));
  }

  function addPayment() {
    setPayments((current) => [...current, { method: "pix", amount: "" }]);
  }

  function removePayment(index: number) {
    setPayments((current) => current.filter((_, paymentIndex) => paymentIndex !== index));
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError("");
    if (summary.remaining_cents > 0) {
      setError(`Ainda falta ${money(summary.remaining_cents)} para fechar a comanda.`);
      return;
    }
    const paymentLines = payments
      .map((payment) => ({ method: payment.method, amount_cents: amountToCents(payment.amount) }))
      .filter((payment) => payment.amount_cents > 0);
    if (paymentLines.length === 0) {
      setError("Informe ao menos uma forma de pagamento.");
      return;
    }
    try {
      await api.checkout({
      appointment_id: appointment.id,
      payment_method: paymentLines[0].method,
      paid_cents: summary.paid_cents,
      tip_cents: 0,
      discount_cents: 0,
      payments: paymentLines,
      });
      onSaved();
      onClose();
    } catch (caught) {
      setError((caught as Error).message);
    }
  }
  return (
    <Modal title="Fechamento de Comanda" onClose={onClose}>
      <form className="form-grid" onSubmit={submit}>
        <div className="card full">
          <p className="eyebrow">{appointment.client_name} - {appointment.barber_name}</p>
          <h3>{appointment.services}</h3>
          <div className="toolbar"><span>Subtotal</span><span>{money(appointment.total_cents)}</span></div>
          <div className="toolbar"><span>Total pago</span><span>{money(summary.paid_cents)}</span></div>
          <div className="toolbar"><span>Gorjeta automática</span><span>{money(summary.tip_cents)}</span></div>
          <div className="toolbar"><strong>Falta</strong><strong className="price">{money(summary.remaining_cents)}</strong></div>
        </div>
        <div className="card full">
          <div className="toolbar">
            <h3>Formas de pagamento</h3>
            <button type="button" className="btn ghost compact" onClick={addPayment}><Icon name="adicionar" /> Adicionar</button>
          </div>
          {payments.map((payment, index) => (
            <div className="row full" style={{ alignItems: "end" }} key={`${payment.method}-${index}`}>
              <div className="field">
                <label className="label">Forma</label>
                <select value={payment.method} onChange={(event) => updatePayment(index, { method: event.target.value })}>
                  {paymentOptions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}
                </select>
              </div>
              <div className="field">
                <label className="label">Valor</label>
                <input
                  type="number"
                  min="0.01"
                  step="0.01"
                  value={payment.amount}
                  onChange={(event) => updatePayment(index, { amount: event.target.value })}
                  required
                />
              </div>
              <button type="button" className="btn ghost compact" disabled={payments.length === 1} onClick={() => removePayment(index)}>
                <Icon name="excluir" /> Remover
              </button>
            </div>
          ))}
        </div>
        {error && <p className="form-error full">{error}</p>}
        <div className="row full"><button className="btn primary">Finalizar Pagamento</button><button type="button" className="btn ghost" onClick={onClose}>Cancelar</button></div>
      </form>
    </Modal>
  );
}

function ProfessionalCommissions({ barberId, professional = false }: { barberId: number; professional?: boolean }) {
  const [commissions, setCommissions] = useState<Commission[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let active = true;
    api.commissions(barberId)
      .then((items) => {
        if (active) setCommissions(items);
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [barberId]);

  if (loading) return <div className="skeleton" />;

  const summary = commissionSummary(commissions);
  return (
    <section className="grid">
      {professional && (
        <div className="grid cols-3">
          <Metric label="Serviços" value={`${summary.services}`} icon="servicos" />
          <Metric label="Média" value={`${summary.average_percent}%`} icon="grafico" />
          <Metric label="Retorno estimado" value={money(summary.estimated_return_cents)} icon="dinheiro" />
        </div>
      )}
      {commissions.length === 0 ? (
        <EmptyState title="Sem comissões configuradas" body="As comissões aparecem aqui quando o salão cadastrar serviços para você." />
      ) : (
        <article className="card">
          <table className="table">
            <thead><tr><th>Serviço</th><th>Preço</th><th>Comissão</th><th>Retorno estimado</th></tr></thead>
            <tbody>
              {commissions.map((commission) => (
                <tr key={commission.service_id}>
                  <td>{commission.service_name}</td>
                  <td>{money(commission.price_cents)}</td>
                  <td>{commission.commission_percent}%</td>
                  <td className="price">{money(commission.estimated_return_cents)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </article>
      )}
    </section>
  );
}

function CommissionsModal({ barber, onClose, onSaved }: { barber: Barber; onClose: () => void; onSaved: () => void }) {
  const [commissions, setCommissions] = useState<Commission[]>([]);
  useEffect(() => { api.commissions(barber.id).then(setCommissions); }, [barber.id]);
  return (
    <Modal title={`Comissões - ${barber.name}`} onClose={onClose} large>
      <table className="table">
        <thead><tr><th>Serviço</th><th>Preço</th><th>Comissão</th><th>Retorno</th></tr></thead>
        <tbody>
          {commissions.map((commission) => (
            <tr key={commission.service_id}>
              <td>{commission.service_name}</td>
              <td>{money(commission.price_cents)}</td>
              <td>
                <input
                  type="number"
                  min={0}
                  max={100}
                  defaultValue={commission.commission_percent}
                  onBlur={async (event) => {
                    const updated = await api.updateCommission(barber.id, { service_id: commission.service_id, commission_percent: Number(event.target.value) });
                    setCommissions(updated);
                    onSaved();
                  }}
                /> %
              </td>
              <td className="price">{money(commission.estimated_return_cents)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </Modal>
  );
}

function Modal({ title, children, onClose, large }: { title: string; children: ReactNode; onClose: () => void; large?: boolean }) {
  return (
    <div className="modal-backdrop" role="dialog" aria-modal="true" onMouseDown={onClose}>
      <section className={`modal ${large ? "large" : ""}`} onMouseDown={(event) => event.stopPropagation()}>
        <div className="toolbar">
          <h3>{title}</h3>
          <button className="btn ghost" onClick={onClose}><Icon name="fechar" /></button>
        </div>
        {children}
      </section>
    </div>
  );
}

function PhoneField({ name, label, defaultValue = "" }: { name: string; label: string; defaultValue?: string }) {
  return (
    <div className="field">
      <label className="label">{label}</label>
      <input
        name={name}
        type="tel"
        defaultValue={formatMobilePhoneInput(defaultValue)}
        maxLength={16}
        placeholder="(11) 99999-8888"
        title="Informe um celular com DDD e 9 dígitos ou uma sequência repetida, como (999) 99999-9999."
        required
        onInput={(event) => {
          event.currentTarget.value = formatMobilePhoneInput(event.currentTarget.value);
        }}
      />
    </div>
  );
}

function Field({ name, label, defaultValue, type = "text", full, required = true, onInput, pattern, title, placeholder }: {
  name: string;
  label: string;
  defaultValue?: string;
  type?: string;
  full?: boolean;
  required?: boolean;
  onInput?: (value: string) => void;
  pattern?: string;
  title?: string;
  placeholder?: string;
}) {
  return (
    <div className={`field ${full ? "full" : ""}`}>
      <label className="label">{label}</label>
      <input
        name={name}
        type={type}
        defaultValue={defaultValue}
        onInput={(event) => onInput?.(event.currentTarget.value)}
        required={required}
        pattern={pattern}
        title={title}
        placeholder={placeholder}
      />
    </div>
  );
}

function Select({ name, label, options, defaultValue }: { name: string; label: string; options: Array<[number | string, string]>; defaultValue?: number | string }) {
  return (
    <div className="field">
      <label className="label">{label}</label>
      <select name={name} defaultValue={defaultValue} required>
        {options.map(([value, labelText]) => <option value={value} key={value}>{labelText}</option>)}
      </select>
    </div>
  );
}
