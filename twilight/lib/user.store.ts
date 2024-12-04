import { Account, Actor } from "./models";
import { makeAutoObservable } from "mobx";

export class UserStore {
    actor: Actor | null = null;
    account: Account | null = null;

    constructor() {
        makeAutoObservable(this);
    }
}
