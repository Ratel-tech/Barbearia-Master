# stitch_barbershop_management_system

Sistema completo de gerenciamento de barbearia com backend em Rust e frontend em React/Vite.

## Status do projeto

Aplicação funcional para uso do salão e do profissional, com:

- autenticação por tipo de conta;
- portal do profissional com agenda e comissões;
- cadastro de clientes, serviços, profissionais e agendamentos;
- fechamento de comanda com pagamentos divididos e gorjeta automática;
- financeiro básico;
- reset de senha por token, com suporte a SMTP ou modo local de desenvolvimento;
- PWA instalável no celular.

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

O banco SQLite e criado automaticamente em `backend/database.db` com seeds de desenvolvimento.

## Scripts da raiz

```powershell
npm run build
npm run lint
npm run api:test
npm run check
```

## Funcionalidades

- Login com selecao entre estabelecimento e profissional.
- Agenda visual com atendimentos reais vindos do backend.
- Cadastro e busca de clientes com validacoes de telefone e CPF.
- Catalogo de servicos com cadastro, edicao e ativacao/desativacao.
- Cadastro, listagem e comissoes de profissionais.
- Fechamento de comanda com multiplas formas de pagamento e gorjeta automatica quando o valor pago excede o subtotal.
- Financeiro basico calculado a partir dos pagamentos e das despesas.
- Portal mobile do profissional com agenda e comissoes.
- Reset de senha com entrega por e-mail em SMTP ou modo local para desenvolvimento.
- PWA com instalacao no celular e icone do aplicativo.

## Observacoes

- O projeto usa SQLite por padrao no desenvolvimento.
- A configuracao de reset de senha por e-mail depende das variaveis documentadas no `.env.example`.
- Os arquivos de planejamento e contexto interno ficam ignorados no repositório.
