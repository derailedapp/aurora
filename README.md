# Derailed

A social media where you're the platform. Constantly-evolving Protocol for Decentralized communication.
Social networking, instant messaging, and more, pick your poison, and keep your data, yours.

## Architecture

Derailed is built on the home-made Aurora Protocol. The Protocol defines a set of behaviors and gateways
which allow Derailed to do decentralized instant messaging, social networking, and more securely. These are the
parts:

- Rail Depot - The core piece of the Derailed decentralized question.
  Rail Depot handles the identification of users, guilds, and dm channels. It is a centralized, easy-to-run, cheap to run
  service which allows you to switch Persona servers without leaving behind your identity.

- Persona - The service and protocol which stores you user data, tracks, direct messages, etc.
  It is easily hostable and requires only one external service.

- Beam - The protocol which provides feeds for Derailed clients to use. While the official Beam implementation will include an algorithmic design
  with an optional fallback to simple by-date or by-following design, this isn't part of the implementation. All Beam defines is an HTTP API which on demand
  returns a list of tracks using on-the-fly sent metadata from Derailed clients. Users get to pick their own Beam feeds as they wish, so you can tune into your
  gaming beam, or if you're feeling spicy the drama beam. The world is in your hands.

- Dialogue - The service used for storing, managing, and distributing Derailed's guilds and lodges.
  Guilds are best compared to Discord Servers and Lodges are best compared to Subreddits. Users pick a Dialogue server to go onto
  and get a list of the guilds and lodges on that server by relevance. Dialogue servers can be configured to be limited to specific Persona servers
  or be fully public and give access to all Persona servers. (The config is split up between `creator_personas` (personas who can *create* guilds and lodges)
  and `participant_personas` (personas who can have their users participating in this Dialogue). These fields can be set none to allow public access.)

## License

Derailed is completely open source. It is licensed under Apache-2.0.
