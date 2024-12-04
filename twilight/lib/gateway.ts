/*
Copyright (C) 2024 V.J. De Chico

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

import AsyncStorage from '@react-native-async-storage/async-storage';
import { backOff } from "exponential-backoff";
import EventEmitter from "eventemitter3"

export class Gateway extends EventEmitter {
    ws: WebSocket | null;
    connected: boolean = false;

    constructor() {
        super();
        this.ws = null;
    }

    connect() {
        if (this.connected) {
            return
        };
        this.connected = true;

        this.ws = new WebSocket(process.env.EXPO_PUBLIC_GATEWAY_URL!);

        this.ws.onopen = async (ev) => {
            // TODO: multiple account login support
            this.ws?.send(JSON.stringify({op: 0, d: {token: await AsyncStorage.getItem("token")!}}));
        };
 
        this.ws.onmessage = async (ev) => {
            let data = JSON.parse(ev.data);

            this.emit(data.t, data.d);
        }

        this.ws.onclose = async (ev) => {
            console.error(`WebSocket server closed with code ${ev.code} (${ev.reason})`);
            this.connected = false;
            // TODO: handle any errors
            await backOff(async () => this.connect(), { maxDelay: 60_000, numOfAttempts: 1000 });
        };
    }
}
