# Como funciona UDP hole punching

Código do artigo de 18 de agosto de 2026. Cada peer recebe manualmente o endpoint público do outro e tenta criar um caminho UDP direto.

## Requisitos

- Rust e Cargo;
- duas máquinas em redes diferentes para testar a travessia de NAT.

Antes do teste, descubra os endereços públicos dos dois peers e troque os valores manualmente. O exemplo assume que ambos os NATs preservam a porta UDP `50000`. A descoberta de outra porta escolhida pelo NAT e a troca automática de endpoints não fazem parte deste exemplo.

## Execução

Na rede de Murilo, informe o endpoint público de Anderson:

```console
$ cargo run --release -- 203.0.113.9:50000
```

Na rede de Anderson, informe o endpoint público de Murilo:

```console
$ cargo run --release -- 198.51.100.7:50000
```

Os dois processos abrem a porta local UDP `50000` e tentam uma troca direta por até dez segundos.

O teste pode falhar conforme o comportamento dos NATs e firewalls envolvidos. Essa falha faz parte do experimento: UDP hole punching não garante conectividade direta em todas as redes.
