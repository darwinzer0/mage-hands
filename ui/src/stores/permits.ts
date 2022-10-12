import { writable } from 'svelte/store';
import ls from 'localstorage-slim';
import encUTF8 from 'crypto-js/enc-utf8.js';
import AES from 'crypto-js/aes.js';
import { Permit } from 'secretjs';

export const permitName = "Mage Hands";

ls.config.encrypt = true;         // global encryption
ls.config.secret = 'font-size';   // global secret

// update encrypter to use AES encryption
ls.config.encrypter = (data, secret) => AES.encrypt(JSON.stringify(data), secret).toString();
 
// update decrypter to decrypt AES-encrypted data
ls.config.decrypter = (data, secret) => {
    try {
        return JSON.parse(AES.decrypt(data, secret).toString(encUTF8));
    } catch (e) {
        // incorrect/missing secret, return the encrypted data instead
        return data;
    }
};

export const permitsStore = writable<object>(ls.get('permits') || {});

permitsStore.subscribe((value: Permit) => {
    ls.set('permits', value);
});