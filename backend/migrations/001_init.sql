pragma foreign_keys = on;

create table if not exists barbershops (
    id integer primary key autoincrement,
    name text not null,
    slug text not null unique,
    created_at text not null default current_timestamp
);

create table if not exists users (
    id integer primary key autoincrement,
    barbershop_id integer references barbershops(id),
    name text not null,
    email text not null unique,
    password_hash text not null,
    role text not null check (role in ('owner', 'admin', 'reception')),
    created_at text not null default current_timestamp
);

create table if not exists clients (
    id integer primary key autoincrement,
    barbershop_id integer references barbershops(id),
    name text not null,
    phone text not null,
    email text,
    document text,
    haircut_frequency text,
    total_spent_cents integer not null default 0,
    visits integer not null default 0,
    deleted_at text
);

create table if not exists barbers (
    id integer primary key autoincrement,
    barbershop_id integer references barbershops(id),
    name text not null,
    document text not null default '',
    email text not null default '',
    password_hash text not null default '',
    specialty text not null default '',
    status text not null default 'active',
    monthly_commission_cents integer not null default 0,
    monthly_tips_cents integer not null default 0,
    completed_services integer not null default 0,
    deleted_at text
);

create table if not exists services (
    id integer primary key autoincrement,
    barbershop_id integer references barbershops(id),
    name text not null,
    description text not null default '',
    duration_minutes integer not null check (duration_minutes > 0),
    price_cents integer not null check (price_cents >= 0),
    category text not null default 'geral',
    active integer not null default 1
);

create table if not exists barber_service_commissions (
    barber_id integer not null references barbers(id) on delete cascade,
    service_id integer not null references services(id) on delete cascade,
    commission_percent integer not null check (commission_percent >= 0 and commission_percent <= 100),
    primary key (barber_id, service_id)
);

create table if not exists appointments (
    id integer primary key autoincrement,
    barbershop_id integer references barbershops(id),
    client_id integer not null references clients(id),
    barber_id integer not null references barbers(id),
    starts_at text not null,
    status text not null check (status in ('scheduled', 'in_chair', 'completed', 'cancelled')),
    total_cents integer not null default 0,
    created_at text not null default current_timestamp
);

create table if not exists appointment_services (
    appointment_id integer not null references appointments(id) on delete cascade,
    service_id integer not null references services(id),
    price_cents integer not null,
    primary key (appointment_id, service_id)
);

create table if not exists payments (
    id integer primary key autoincrement,
    barbershop_id integer references barbershops(id),
    appointment_id integer not null references appointments(id),
    method text not null check (method in ('pix', 'cash', 'debit', 'credit')),
    subtotal_cents integer not null,
    discount_cents integer not null default 0,
    tip_cents integer not null default 0,
    paid_cents integer not null,
    change_cents integer not null,
    total_cents integer not null,
    created_at text not null default current_timestamp
);

create table if not exists payment_splits (
    id integer primary key autoincrement,
    payment_id integer not null references payments(id) on delete cascade,
    method text not null check (method in ('pix', 'cash', 'debit', 'credit')),
    amount_cents integer not null check (amount_cents > 0)
);

create table if not exists extra_expenses (
    id integer primary key autoincrement,
    barbershop_id integer references barbershops(id),
    description text not null,
    amount_cents integer not null check (amount_cents > 0),
    created_at text not null default current_timestamp
);

create table if not exists audit_logs (
    id integer primary key autoincrement,
    barbershop_id integer references barbershops(id),
    actor text not null,
    action text not null,
    entity text not null,
    entity_id integer,
    payload text,
    created_at text not null default current_timestamp
);

create table if not exists sessions (
    token text primary key,
    subject_id integer not null,
    subject_type text not null check (subject_type in ('user', 'barber')),
    barbershop_id integer not null references barbershops(id),
    role text not null check (role in ('owner', 'admin', 'barber', 'reception')),
    barber_id integer references barbers(id),
    created_at text not null default current_timestamp
);

create table if not exists password_reset_tokens (
    token text primary key,
    subject_id integer not null,
    subject_type text not null check (subject_type in ('user', 'barber')),
    email text not null,
    expires_at text not null,
    used_at text,
    created_at text not null default current_timestamp
);

create table if not exists auth_challenge_attempts (
    scope text not null,
    identifier text not null,
    failures integer not null default 0,
    locked_until text,
    last_failed_at text not null default current_timestamp,
    primary key (scope, identifier)
);

create index if not exists idx_users_barbershop_email on users(barbershop_id, email);
create index if not exists idx_clients_name on clients(name);
create index if not exists idx_clients_barbershop on clients(barbershop_id);
create unique index if not exists idx_clients_active_phone_unique on clients(barbershop_id, phone) where deleted_at is null;
create unique index if not exists idx_clients_active_document_unique on clients(barbershop_id, document) where deleted_at is null and document is not null and document != '';
create index if not exists idx_barbers_status on barbers(status);
create index if not exists idx_barbers_barbershop on barbers(barbershop_id);
create unique index if not exists idx_barbers_active_email_unique on barbers(lower(email)) where deleted_at is null and email != '';
create unique index if not exists idx_barbers_active_document on barbers(document) where deleted_at is null and document != '';
create index if not exists idx_services_active on services(active);
create index if not exists idx_services_barbershop on services(barbershop_id);
create index if not exists idx_appointments_starts_at on appointments(starts_at);
create index if not exists idx_appointments_barbershop on appointments(barbershop_id);
create index if not exists idx_payments_created_at on payments(created_at);
create index if not exists idx_payment_splits_payment on payment_splits(payment_id);
create index if not exists idx_extra_expenses_created_at on extra_expenses(created_at);
create index if not exists idx_password_reset_tokens_subject on password_reset_tokens(subject_type, subject_id, used_at);
create index if not exists idx_auth_challenge_attempts_locked_until on auth_challenge_attempts(locked_until);
