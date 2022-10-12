import type { KeplrStore } from '../stores/keplr';
import { keplrStore } from '../stores/keplr';
import { get } from 'svelte/store';

export async function holdForKeplr(keplr: KeplrStore) {
    if (keplr && keplr.scrtAuthorized) { return keplr; }
    while (true) {
        keplr = get(keplrStore);
        if (keplr.scrtAuthorized) {
            return keplr;
        }
        await new Promise(resolve => setTimeout(resolve, 500));
    }
}