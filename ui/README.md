## Mage Hands crowdfunding UI

### Running on testnet

Send some tokens to your address: 

`docker exec -it localsecret secretcli tx bank send a secretaddress... 10000000000uscrt`

Send some sscrt to your address:

`docker exec -it localsecret secretcli tx snip20 send secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg secretaddress... "10000000000" --from a`