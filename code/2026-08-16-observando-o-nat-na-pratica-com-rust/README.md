# Observando o NAT na prática com Rust

Código do artigo de 16 de agosto de 2026. O experimento compara o endpoint local de um socket UDP com a origem identificada por um servidor UDP público.

## Requisitos

- Rust e Cargo;
- uma máquina com IPv4 público para observar a tradução UDP pela internet.

## Teste UDP local

Em um terminal:

```console
$ cargo run --bin server
servidor iniciado em 0.0.0.0:40000
```

Em outro:

```console
$ cargo run --bin client -- 127.0.0.1:40000
endpoint local: 127.0.0.1:50000
destino: 127.0.0.1:40000
origem identificada pelo servidor: 127.0.0.1:50000
```

Para testar entre máquinas da mesma rede, substitua `127.0.0.1` pelo endereço privado da máquina que executa o servidor.

## Servidor atrás do NAT

1. Execute o servidor em uma máquina da rede local.
2. Descubra o endereço público do roteador dessa rede.
3. Em uma máquina de outra rede, execute `cargo run --bin client -- IP_PUBLICO:40000`.

Sem redirecionamento de porta ou estado NAT compatível, o cliente deve atingir o timeout após três segundos e o servidor não deve receber o datagrama. Executar o servidor em `0.0.0.0:40000` não publica automaticamente a porta no roteador.

## Servidor com endereço público

1. Copie o binário `server` para uma máquina com IPv4 público ou compile o projeto nela.
2. Libere a porta UDP `40000` no firewall do sistema e nas regras do provedor.
3. Execute `cargo run --release --bin server` na máquina pública.
4. Execute `cargo run --bin client -- IP_PUBLICO:40000` na máquina atrás do NAT.

O cliente deve mostrar seu endpoint local e a origem informada pelo servidor. Os valores normalmente diferem quando o datagrama atravessa NAPT.