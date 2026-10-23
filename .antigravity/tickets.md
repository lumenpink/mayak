# Active Tickets - mayak
> Synced at: 2026-10-23 08:49:23 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#11] fix(build): otimizar tamanho de binario e habilitar linking estatico musl
- **Status**: `open`
- **Labels**: none
- **Link**: https://git.2lp.in/lumen/mayak/issues/11

### Description
_No description provided._

---
## [#10] feat(mayak): integracao de binarios na banca e atualizador xdg
- **Status**: `open`
- **Labels**: none
- **Link**: https://git.2lp.in/lumen/mayak/issues/10

### Description
Adequar a esteira de CI/CD do repositório para compilar exclusivamente no runner soberano e publicar os artefatos diretamente na Banca (https://banca.2lp.in/downloads/mayak/), mantendo retenção de 5 versões estáveis e 10 nightlies via Cloudflare R2, sem hospedar binários no Codeberg.

Implementar na CLI a funcionalidade de auto-atualização atômica (mayak update e mayak check-update) com estrita conformidade com a especificação OpenDesktop (XDG Base Directory) e o Perfil Lumen da norma GOST:

1. Configuração do usuário em $XDG_CONFIG_HOME/mayak/config.toml (fallback ~/.config/mayak/config.toml).
2. Persistência de cache e estado de verificação em $XDG_STATE_HOME/mayak/update_check.json (fallback ~/.local/state/mayak/update_check.json).
3. Suporte no config.toml a:
   - channel = "stable" | "nightly"
   - pinned = true | false
   - pin = "<versao>"
   - auto_check = true | false
   - check_interval_hours = 24
   - auto_update = false (desabilitado por padrão; quando habilitado e não-pinado, aplica a atualização atômica para o próximo arranque)
4. Verificação periódica não-bloqueante no startup de comandos normais da CLI, com emissão de alerta discreto exclusivamente em stderr caso haja versão mais recente no canal (desde que não esteja pinado).
5. Subcomandos de configuração:
   - mayak config show / mayak config get
   - mayak config pin [<version>]
   - mayak config unpin
   - mayak config set-auto-check <true|false>
   - mayak config set-channel <stable|nightly>
   - mayak config set-auto-update <true|false>
6. Atualização atômica com arquivo temporário no mesmo sistema de arquivos, permissão de execução e substituição não-destrutiva em ~/.local/bin/mayak com backup e rollback automático em caso de erro.

Expected Behavior & Acceptance Criteria:
- Pipeline CI/CD publicando na Banca (https://banca.2lp.in/downloads/mayak/) com retenção 5 stable / 10 nightly.
- Remoção de publicação de binários pesados no Codeberg Releases.
- Configuração em $XDG_CONFIG_HOME/mayak/config.toml e cache em $XDG_STATE_HOME/mayak/update_check.json.
- Respeito à ordem de precedência: flag --channel > env MAYAK_CHANNEL > config.toml > stable.
- Respeito a travamento de versão (pinned = true ou pin), bloqueando updates sem --force e silenciando alertas no startup.
- Checagem rápida não-bloqueante no startup com alerta em stderr.
- Mecanismo de auto_update opcional/configurável para substituição transparente em segundo plano.
- Subcomandos mayak update, mayak check-update e mayak config ... funcionais.
- Atualização atômica do binário em ~/.local/bin/mayak com rollback.

Tasks / DD-TDD Checklist (9 Fases GOST):
- Fase 1 - Ticket: Issue formalizada e escopo delimitado
- Fase 2 - Branch: Branch feat/<id>-updater-opendesktop-xdg criada a partir de main
- Fase 3 - Doc/Spec: Contrato preliminar e especificação (docs/spec-<id>-updater.md)
- Fase 4 - Red: Teste automatizado falho comprovando parsing XDG, channels, auto_update e pinning
- Fase 5 - Green: Implementação mínima funcional com teste passando
- Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings
- Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas
- Fase 8 - Quality Gate: Cobertura >= 90%, linter/clippy e formatação validados
- Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #)

---
*Migrado do Codeberg (#issue-10 por @lumenpink)*

---
