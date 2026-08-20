# O Meu LabNet

Painel pessoal no ecossistema LabNetCol — **Rust + Dioxus 0.7** + **mcs_bd2**.

## Arquitectura

| Componente | Porto | Descrição |
|------------|-------|-----------|
| Frontend (`dx serve`) | **8092** | UI Dioxus (web) |
| API (`meu_labnet_serv`) | **8093** | Axum + mcs_bd2 |

Entidades BD: `painel` (preferências) e `widget` (atalhos, links, notas) por `utilizador` (n_reg LabNetCol).

## Desenvolvimento

```bash
# Terminal 1 — API + BD
cd server
MEU_LABNET_DATA_DIR=../data cargo run -p dio-meu-lab-net-server

# Terminal 2 — UI
dx serve --port 8092
```

LabNetCol em `:8080` (SSO). Variáveis úteis:

```bash
LABNETCOL_SECRET=labnetcol-sso-dev-secret   # igual ao LabNetCol
MEU_LABNET_JWT_SECRET=meu-labnet-jwt-dev
MEU_LABNET_DEV_LOGIN=1
```

## API (resumo)

- `POST /api/sso/labnetcol` — troca JWT LabNetCol por token da app
- `GET /api/painel` — config + widgets do utilizador
- `PUT /api/painel/config` — colunas, tema
- `POST /api/widgets` — novo link/nota
- `DELETE /api/widgets/{id}` — remover

Logout: redirect portal `/?logout=1`.
