# Especificação Técnica: Otimização de Binário e Linking Estático musl

- **Ticket**: #11
- **Escopo**: Configuração de LTO fat no perfil `[profile.release]` e flags de linking estático com musl libc no microservidor `mayak` (`mk`).

## 1. Contexto e Motivação
A configuração atual utilizava LTO thin (`lto = true`) e não contava com `.cargo/config.toml` contendo `-C target-feature=+crt-static`. Isso deixava o binário sujeito a carregamento dinâmico em ambientes musl e não atingia o teto máximo de compressão estática.

## 2. Requisitos Técnicos
1. `Cargo.toml`:
   - `opt-level = 3`.
   - `lto = "fat"`.
   - `codegen-units = 1`.
   - `panic = "abort"`.
   - `strip = true`.
2. `.cargo/config.toml`:
   - Configurar `rustflags = ["-C", "target-feature=+crt-static"]` para os targets `x86_64-unknown-linux-musl` e `aarch64-unknown-linux-musl`.
