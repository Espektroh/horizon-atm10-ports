# glue_mockcheck

Verificação de tipos do `glue.rs` sem o Horizon: `horizon_world`, `horizon_core` e `horizon_client` aqui são
imitações mínimas com as mesmas assinaturas que o `glue.rs` usa (conferidas no código do Horizon 66f3981, ver
`APIS.md`). Compila `runtime.rs` + `glue.rs` como `crate::game::powah_native` / `powah_glue` (e os 8 helpers como funções), como o
horizon-mixins faz. Pega erro de sintaxe, `match` incompleto e tipo errado na lógica do mod; NÃO prova que a
assinatura real do Horizon é a imitada (isso é o `APIS.md`).

    CARGO_TARGET_DIR=/tmp/powah_mc cargo build --offline
