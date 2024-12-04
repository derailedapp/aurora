export const registerUser = (username: string, password: string, email: string) => {
    return fetch(process.env.EXPO_PUBLIC_API_URL! + "/register", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({ username: username, password: password, email: email }),
    });
};

export const loginUser = (password: string, email: string) => {
    return fetch(process.env.EXPO_PUBLIC_API_URL! + "/login", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({ password: password, email: email }),
    });
};
