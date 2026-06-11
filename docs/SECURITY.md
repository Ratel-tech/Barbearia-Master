# Security Notes

## Ja aplicado

- Queries SQL parametrizadas via SQLx.
- Constraints relacionais e checks no schema.
- CORS explicito no backend para ambiente de desenvolvimento.
- Limite de tamanho de request no backend.
- Tratamento de erro centralizado sem expor stack traces.
- Valores monetarios inteiros em centavos.
- Soft delete preparado em clientes e profissionais.
- Protecao de login e recuperacao de senha com hCaptcha apos tentativas repetidas.

## Proximos endurecimentos antes de producao

- Trocar `CorsLayer::permissive()` por allowlist de dominios.
- Implementar autenticacao completa com Argon2, refresh tokens e expiracao.
- Implementar CSRF se a autenticacao migrar para cookies.
- Adicionar RBAC por rota.
- Persistir audit logs em todos os fluxos sensiveis.
- Rate limiting por IP/usuario.
- Segredos via cofre/variaveis de ambiente.
- Nunca publicar `.env`, bancos locais, logs, arquivos de agente ou senhas demo. `SEED_DEMO_DATA=1` exige `DEMO_ADMIN_PASSWORD` definido fora do Git.
- PostgreSQL gerenciado com backups e TLS.
