# glue_mockcheck

Verificação de tipos do `glue.rs` sem o Horizon: `horizon_world`, `horizon_core` e `horizon_client` aqui são
imitações mínimas com as mesmas assinaturas que o `glue.rs` usa (conferidas no código do Horizon 66f3981, ver
`APIS.md`). Compila `runtime.rs` + `glue.rs` como `crate::game::mekanism_native` / `mekanism_glue`, como o
horizon-mixins faz. Pega erro de sintaxe, `match` incompleto e tipo errado na lógica do mod; NÃO prova que a
assinatura real do Horizon é a imitada (isso é o `APIS.md`).

    CARGO_TARGET_DIR=/tmp/mc cargo build --offline
