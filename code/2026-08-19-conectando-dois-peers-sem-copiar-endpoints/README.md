# Conectando dois peers sem copiar endpoints

Código do artigo de 19 de agosto de 2026. Dois peers se registram em uma sala do rendezvous, recebem os candidates um do outro e tentam abrir um caminho UDP direto sem que ninguém copie um endpoint à mão.

São dois binários em um único crate:

- `rendezvous`: servidor TCP que apresenta os dois peers de uma sala e fecha a conexão;
- `peer`: consulta STUN, registra-se na sala e faz UDP hole punching com os candidates recebidos.

## Requisitos

- Rust e Cargo;
- acesso à internet para a consulta STUN a `stun.cloudflare.com:3478`;
- para testar a travessia de NAT de verdade, duas máquinas em redes diferentes e um rendezvous alcançável pelas duas.

## Execução

Primeiro o rendezvous:

```console
$ cargo run --release --bin rendezvous
rendezvous escutando em 0.0.0.0:7777
```

Depois cada peer, em seu próprio terminal, usando o mesmo nome de sala:

```console
$ cargo run --release --bin peer -- murilo demonstracao 127.0.0.1:7777
```

```console
$ cargo run --release --bin peer -- anderson demonstracao 127.0.0.1:7777
```

Quando os dois entram na sala, o rendezvous troca os candidates e sai do caminho. As linhas digitadas depois disso viajam direto entre os peers, por UDP. `/quit` encerra a sessão e avisa o outro lado.

Executando as duas pontas na mesma máquina, o caminho escolhido costuma ser o de loopback. Para atravessar NAT, publique o rendezvous em um host alcançável pelas duas redes e passe o endereço dele no terceiro argumento.

## Limites

O caminho direto pode não existir. Se nenhum candidate responder em dez segundos, o peer informa a falha e encerra: não há TURN nem qualquer outra forma de relay aqui.

Não há entrega garantida, ordenação, criptografia ou autenticação. Os datagramas são texto puro, e quem souber o nome da sala pode se registrar nela.
