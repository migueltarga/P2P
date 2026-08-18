# Como descobrir seu endpoint externo com STUN

Código do artigo de 18 de agosto de 2026. O programa consulta `stun.cloudflare.com:3478`, imprime o endpoint externo visto pelo NAT, espera a troca manual com o outro peer e usa o mesmo socket para tentar UDP hole punching.

## Execução

Em cada peer:

```console
$ cargo run --release
socket local: 0.0.0.0:50000
endpoint externo: 198.51.100.7:62000
envie o endpoint externo ao outro peer
endpoint externo do outro peer:
```

Murilo envia seu endpoint externo a Anderson, Anderson envia o dele a Murilo, e cada um cola o valor recebido no terminal.

`0.0.0.0:50000` indica que o socket aceita datagramas em qualquer interface IPv4 local. Esse não é o valor que deve ser compartilhado; o endpoint externo retornado por STUN é a informação útil para a tentativa.

O socket precisa permanecer aberto entre a consulta STUN e as tentativas de hole punching. O NAT pode mapear a porta local para uma externa diferente, e isso é exatamente o que a consulta mede.

O teste ainda pode falhar por causa de mapping, filtering, múltiplas camadas de NAT ou políticas de firewall. STUN descobre uma possibilidade de endereço; não garante conectividade com o outro peer.