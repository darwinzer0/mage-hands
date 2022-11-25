## Mage Hands crowdfunding UI

### Pulsar

Pulsar testnet version can be found online at https://pulsar.catallaxy.fund.

Platform contract address: secret1tffa8awm3ycup7a2up5gsrmn99mpcp53myz9v2

SSCRT contract address: secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg

### Running on testnet

Send some tokens to your address: 

`docker exec -it localsecret secretcli tx bank send a secretaddress... 10000000000uscrt`

Send some sscrt to your address:

`docker exec -it localsecret secretcli tx snip20 send secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg secretaddress... "10000000000" --from a`