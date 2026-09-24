# Mayak (mk) — Маяк

> **Micro static HTTP server in pure Rust**  
> *Minimalist · Multi-threaded · Byte-Range (206) Streaming · Gosplan Radio & Static Distribution (Маяк)*

[![CI](https://codeberg.org/lumenpink/mayak/actions/workflows/ci.yml/badge.svg)](https://codeberg.org/lumenpink/mayak/actions)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE)
[![Website](https://img.shields.io/badge/docs-mayak.2lp.in-purple)](https://mayak.2lp.in)

---

## 1. Visão Geral

O **Mayak** (e seu atalho imediato **`mk`**) é um micro-servidor HTTP estático desenvolvido em **Rust**, concebido para servir documentação, sites estáticos, SPAs, bundles Wasm e mídia de alta performance com footprint mínimo, velocidade instantânea e zero atrito de configuração.

O nome homenageia a histórica emissora estatal soviética **Rádio Mayak** (*Радиостанция «Маяк»* — Farol), célebre por sua transmissão contínua e confiável para toda a União.

### Por que o Mayak?
* **Zero Configuração:** Serve `./site/`, `./dist/` ou o diretório atual de forma inteligente e imediata.
* **Rede Acessível por Padrão:** Faz bind em `0.0.0.0` por padrão, permitindo testes remotos via Wi-Fi local, Tailscale ou VMs, sem exigir flags adicionais.
* **Auto-descoberta de Portas:** Se a porta padrão `8080` estiver em uso, busca e ocupa automaticamente a próxima porta livre (`8081`, `8082`...).
* **Suporte Completo a Range Requests (`206 Partial Content`):** Suporte nativo a `Range: bytes=start-end` para streaming e seeking contínuo de áudio e vídeo (essencial para ecossistemas como o [La Craia](https://codeberg.org/lumenpink/lacraia)).
* **Multi-thread e Concorrência Nativa:** Atende dezenas de requisições simultâneas sem travar a interface.
* **Single Page Application (`-s, --spa`):** Redireciona rotas inexistentes (404) para `index.html`.
* **Binário Único & Compacto:** Compilado em binário estático único (~1.3 MB) sem dependências pesadas de runtime.

---

## 2. Instalação

### 2.1 A partir do Código-Fonte (Cargo)
```bash
git clone ssh://git@codeberg.org/lumenpink/mayak.git
cd mayak
cargo install --path .
```
Isso instalará os binários **`mayak`** e **`mk`** no seu `~/.cargo/bin` (ou `~/.local/bin`).

### 2.2 Compilação Manual Rápida
```bash
cargo build --release
cp target/release/mk ~/.local/bin/mk
cp target/release/mayak ~/.local/bin/mayak
```

---

## 3. Uso & Exemplos

```bash
mk [opções] [diretório]
# ou
mayak [opções] [diretório]
```

### 3.1 Exemplos Rápidos

```bash
# Servir a pasta atual em 0.0.0.0:8080 (ou próxima porta livre)
mk

# Servir uma pasta de documentação específica e abrir no navegador
mk site/ --open

# Servir uma SPA na porta 3000 com CORS liberado
mk dist/ -p 3000 --spa --cors

# Restringir o acesso estritamente à máquina local (localhost)
mk --localhost

# Modo silencioso (suprime logs de requisições individuais)
mk -q
```

### 3.2 Tabela de Opções

| Opção | Atalho | Descrição |
| :--- | :--- | :--- |
| `-p, --port <N>` | `-p 8080` | Porta de escuta inicial (incrementa se ocupada) |
| `--host <IP>` | `--host 0.0.0.0` | Endereço IP da interface de rede (padrão: `0.0.0.0`) |
| `-l, --localhost` | `-l` | Atalho para escutar estritamente em `127.0.0.1` |
| `-s, --spa` | `-s` | Redireciona 404 para `index.html` (Single Page Apps) |
| `--cors` | `--cors` | Envia cabeçalhos `Access-Control-Allow-Origin: *` |
| `--bridge <CMD>` | `--bridge cccp` | Habilita Command Bridge travado no binário especificado |
| `--auth <TOKEN>` | `--auth segredo` | Exige token Bearer ou X-Mayak-Token para requisições no bridge |
| `-o, --open` | `-o` | Abre automaticamente o navegador (`xdg-open` / `open`) |
| `-q, --quiet` | `-q` | Modo silencioso (apenas erros são exibidos) |
| `-h, --help` | `-h` | Exibe a mensagem de ajuda |
| `-v, --version` | `-v` | Exibe a versão instalada |

---

## 4. Cobertura de MIME Types

O Mayak inclui tabela interna exaustiva para aplicações modernas:
- **Áudio:** `mp3`, `ogg`, `oga`, `opus`, `flac`, `wav`, `m4a`, `aac`, `weba`, `mid`, `midi`
- **Vídeo:** `mp4`, `m4v`, `webm`, `ogv`, `mkv`, `mov`
- **Web & Wasm:** `html`, `css`, `js`, `mjs`, `wasm`, `json`, `map`, `webmanifest`
- **Fontes:** `woff`, `woff2`, `ttf`, `otf`, `eot`
- **Imagens:** `svg`, `png`, `jpg`, `jpeg`, `webp`, `gif`, `ico`, `avif`
- **Documentos:** `pdf`, `txt`, `md`, `csv`

---

## 5. Integração com o CCCP

O Mayak é o servidor estático prioritário do utilitário [CCCP](https://codeberg.org/lumenpink/cccp):
- Ativado como motor padrão do comando `cccp serve`
- Recrutado automaticamente através do despachante `_cccp_ensure_comrade`

---

## 6. Licença

Este projeto é software livre licenciado sob a [Apache License 2.0](LICENSE).
