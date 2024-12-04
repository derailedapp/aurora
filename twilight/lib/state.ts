import { makeAutoObservable } from "mobx";
import { UserStore } from "./user.store";
import { Gateway } from "./gateway";

class State {
    public user: UserStore;
    public gw: Gateway;

    constructor() {
        makeAutoObservable(this)
        this.user = new UserStore();
        this.gw = new Gateway();
    }
}

export const state = new State();
