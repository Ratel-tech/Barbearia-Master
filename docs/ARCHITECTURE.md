# Architecture

## Objetivo

Transformar as paginas estaticas exportadas pelo Google Stitch em uma aplicacao real, mantendo o visual existente como contrato de produto.

## Monorepo

- `frontend`: interface React. A composicao visual segue o prototipo: navegacao lateral, dark luxury, cards metricos, agenda, modais e fluxo de comanda.
- `backend`: API Rust. O codigo esta separado em `app` para rotas/use cases HTTP, `models` para contratos, `db` para infraestrutura e `error` para tratamento centralizado.
- `.planning`: contexto, requisitos e roadmap do projeto.
- `docs`: decisao tecnica e seguranca.

## Backend

O backend usa Axum e SQLx com queries parametrizadas. A estrutura atual prioriza uma API pequena e clara para as telas existentes:

- `GET /api/overview`
- `GET|POST /api/clients`
- `GET|POST /api/barbers`
- `GET|PUT /api/barbers/{id}/commissions`
- `GET|POST /api/services`
- `GET|POST /api/appointments`
- `POST /api/checkouts`
- `POST /api/auth/login`

SQLite e usado para rodar localmente sem infraestrutura externa. A modelagem preserva relacoes e constraints para facilitar migracao futura para PostgreSQL.

## Banco

Tabelas principais:

- `users`
- `clients`
- `barbers`
- `services`
- `barber_service_commissions`
- `appointments`
- `appointment_services`
- `payments`
- `audit_logs`

Valores monetarios sao armazenados em centavos para evitar erro de ponto flutuante.

## Frontend

O frontend e uma SPA que renderiza apenas as areas presentes no prototipo:

- Agenda
- Clientes
- Servicos
- Financeiro
- Profissionais
- Modais de cadastro, agendamento, comissao e checkout

Os componentes usam estados de loading, mensagens amigaveis e formularios integrados aos endpoints.
