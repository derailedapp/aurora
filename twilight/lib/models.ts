
export interface Actor {
    id: string;
    server_id: string | null;
    username: string;
    display_name: string | null;
    avatar_url: string | null;
    banner_url: string | null;
    bio: string | null;
}

export interface Account {
    id: string,
    actor_id: string,
    email: string | null,
    flags: number | null
}
