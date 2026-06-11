# Estabelecimento: fechamento diário, comissão mensal e barbeiro de teste

## Objetivo

Adicionar ao portal do estabelecimento três capacidades relacionadas ao financeiro e à validação da interface:

1. criar um barbeiro de demonstração para teste do portal do profissional;
2. fechar o caixa do dia com geração de PDF;
3. controlar o fechamento mensal de comissão dos barbeiros com status `open` ou `paid`.

O escopo é do estabelecimento. O profissional continua com acesso somente ao que já lhe pertence, sem ganhar permissão de liquidar comissão ou fechar caixa.

## Estado atual

O sistema já possui:

- autenticação por tipo de conta;
- agenda, checkout e pagamentos divididos;
- cálculo de comissão por serviço;
- financeiro consolidado no `overview`;
- portal mobile do profissional;
- seed de admin de demonstração.

O que ainda falta para este pedido:

- um barbeiro de teste previsível para login do portal profissional;
- um registro de fechamento diário com exportação em PDF;
- uma entidade persistente para comissão mensal paga/não paga.

## Decisões de produto

### 1. Barbeiro de teste

O seed de desenvolvimento vai criar um barbeiro ativo adicional, ligado à mesma barbearia demo do admin.

Regras:

- só existe quando `SEED_DEMO_DATA=1`;
- usa credenciais fixas e documentadas no `.env.example`;
- é ativo por padrão para validar a interface profissional;
- não substitui barbeiros reais já cadastrados.

Credenciais de demonstração:

- e-mail: `barber@example.test`
- senha: `TestPassword@123`

### 2. Fechamento diário em PDF

O fechamento diário não será só um número solto na tela. Ele vira um registro persistido com:

- data do fechamento;
- total recebido no dia;
- soma por forma de pagamento: `cash`, `pix`, `debit`, `credit`;
- descontos;
- gorjetas;
- despesas extras do dia;
- lucro líquido do dia;
- quantidade de atendimentos e pagamentos.

O PDF será gerado no backend, a partir desse registro, com layout simples de relatório operacional.

### 3. Comissão mensal com status

O status `pago`/`não pago` não deve ficar preso apenas em `barbers.monthly_commission_cents`, porque isso mistura saldo acumulado com histórico de liquidação.

A solução será criar um ledger de liquidação mensal por barbeiro e competência:

- competência no formato `YYYY-MM`;
- valor calculado da comissão do mês;
- status `open` ou `paid`;
- data de pagamento quando liquidado;
- referência ao barbeiro e à barbearia.

Isso preserva histórico e permite consultar meses anteriores sem perder o estado atual.

## Arquitetura proposta

### Backend

Adicionar dois novos conjuntos de dados e endpoints:

1. `daily_closings`
   - armazena o fechamento do caixa do dia;
   - gera e serve o PDF;
   - evita recalcular o mesmo fechamento toda vez que o usuário baixar o relatório.
   - possui unicidade por `barbershop_id + closing_date`.

2. `commission_settlements`
   - armazena o fechamento mensal por barbeiro;
   - permite marcar como pago ou retornar para aberto;
   - guarda competência e valor calculado.
   - possui unicidade por `barbershop_id + barber_id + competence`.

### Endpoints propostos

- `GET /api/reports/daily-closures?date=YYYY-MM-DD`
  - retorna pré-visualização do fechamento do dia.
- `POST /api/reports/daily-closures`
  - cria o fechamento persistido para a data informada;
  - se o fechamento já existir para a mesma data e barbearia, retorna o registro existente.
- `GET /api/reports/daily-closures/{id}/pdf`
  - baixa o PDF do fechamento.
- `GET /api/commission-settlements?competence=YYYY-MM`
  - lista os fechamentos mensais de comissão da competência.
- `POST /api/commission-settlements/close-month`
  - gera os registros da competência para os barbeiros ativos;
  - se uma liquidação já existir para a mesma competência, barbeiro e barbearia, reaproveita o registro existente em vez de criar duplicata.
- `PUT /api/commission-settlements/{id}`
  - atualiza `status` e `paid_at`.

### Frontend

No estabelecimento, a área financeira vai ganhar:

- cartão/ação para `Fechar caixa do dia`;
- visualização do resumo antes de gerar o PDF;
- botão de download do PDF;
- tabela de comissão mensal por barbeiro com status;
- ação para marcar comissão como paga e desfazer pagamento;
- indicador do barbeiro de teste disponível para validar o portal profissional.

### Seed de demonstração

O seed vai criar:

- o admin demo atual;
- um barbeiro demo ativo;
- comissão padrão por serviço para esse barbeiro.

Credenciais de demonstração devem ficar documentadas no `.env.example` e no README.

## Regras de cálculo

### Fechamento diário

O resumo do dia deve considerar:

- pagamentos realizados na data informada;
- soma por forma de pagamento real da `payment_splits`;
- gorjetas;
- despesas extras cadastradas na mesma data;
- total bruto, total líquido e lucro final.

O layout do PDF deve mostrar o resumo primeiro e depois os totais por forma de pagamento.

### Comissão mensal

O fechamento mensal deve calcular a comissão a partir de pagamentos concluídos no mês, agrupados por barbeiro.

Regras:

- usar a competência solicitada;
- considerar apenas atendimentos válidos para comissão;
- o valor fechado não deve mudar quando o status virar `paid`;
- o status só muda o estado de liquidação, não o cálculo histórico.

## Segurança e autorização

- Apenas `owner`, `admin` e, se aplicável, `reception` podem fechar caixa e liquidar comissão.
- O barbeiro logado pode ver apenas os próprios dados, sem ações de liquidação.
- O PDF não deve expor dados fora da barbearia autenticada.
- O seed de demo não pode vazar para produção.

## Testes

### Backend

- cria barbeiro demo com credenciais previsíveis quando o seed está ativado;
- gera fechamento diário com totais corretos por método;
- cria PDF de fechamento diário com conteúdo básico verificável;
- cria e lista liquidações mensais por competência;
- alterna comissão mensal entre `open` e `paid`;
- preserva isolamento por barbearia.

### Frontend

- exibe ação de fechamento diário na tela do estabelecimento;
- exibe download do PDF depois do fechamento;
- lista comissão mensal com status;
- permite marcar e desmarcar pagamento;
- mostra barbeiro de teste no portal profissional.

## Fora de escopo

- não trocar o banco para outro SGBD nesta entrega;
- não enviar PDF por e-mail;
- não automatizar o fechamento diário em cron;
- não alterar a lógica de checkout já existente além do que o relatório precisa ler;
- não substituir o fluxo atual de comissão por serviço.

## Critério de pronto

A entrega só estará pronta quando:

- o seed criar um barbeiro de teste funcional;
- o estabelecimento conseguir fechar o dia e baixar um PDF;
- o arquivo seja baixado com nome coerente com a data do fechamento, como `fechamento-caixa-YYYY-MM-DD.pdf`;
- a comissão mensal aparecer com status aberto ou paga;
- o status persistir no banco e não depender de recálculo da tela;
- os testes cobrirem os novos fluxos;
- `npm run check` passar.
