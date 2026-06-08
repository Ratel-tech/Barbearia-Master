# BARBEARIA MASTER

Sistema de gerenciamento de barbearia baseado nos exports estaticos do Google Stitch.

## Estrutura simples

```text
backend/
  migrations/
  src/
  Cargo.toml
  Cargo.lock
  database.db

frontend/
  public/
  src/
  package.json
  vite.config.ts
```

Os prototipos originais do Stitch continuam na raiz como referencia visual.

## Executar

Instalar dependencias do frontend:

```powershell
cd frontend
npm install
```

Rodar backend:

```powershell
cd backend
cargo run
```

Rodar frontend:

```powershell
cd frontend
npm run dev
```

URLs padrao:

- Web: `http://127.0.0.1:5173`
- API: `http://127.0.0.1:8080`
- Healthcheck: `http://127.0.0.1:8080/health`

O banco SQLite e criado automaticamente em `backend/database.db` com seeds de desenvolvimento alinhados ao prototipo.

## Scripts da raiz

```powershell
npm run build
npm run lint
npm run api:test
npm run check
```

## Funcionalidades

- Agenda visual com atendimentos reais vindos do backend.
- Cadastro e busca de clientes.
- Catalogo de servicos com cadastro.
- Cadastro/listagem de profissionais.
- Configuracao de comissao por servico.
- Fechamento de comanda com gorjeta, forma de pagamento e calculo de troco.
- Financeiro basico calculado a partir dos pagamentos.
