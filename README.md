# Welcome to the Hangrier Games! <!-- rumdl-disable-line MD026 -->

This app allows you to play "Hunger Games", as text and emoji, with your
friends, family, coworkers, enemies, or strangers. Or just by yourself. Each
"Game" has a set of 24 contestants, all randomly generated with various names,
statistics, and equipment. The "Tributes" are written to be fairly bloodthirsty,
so fights will be often, and often deadly. Eventually (probably before the heat
death of the universe), the game will end with either one or zero winners. Enjoy
your victory or defeat, and then start a new game!

## FAQs

Here are the questions I often find myself asking about this project:

### Is this over-engineered, being set up as a workspace instead of just a normal crate? <!-- rumdl-disable-line MD013 -->

Yes, probably. But it also makes it easier, at least in my mind, to
have multiple binaries that I could, conceivably, update independently.
Have I done this? No, I don't think so.

### Where the heck does everything live? I can't find anything!

The project is divided into a few crates. Why? See the first question.

- The `game` crate has all of the code that handles game logic.
- The `shared` crate has code that's needed by both the frontend and the API.
  Mostly structs and enums.
- The `api` crate has code that creates the API and handles interaction with the
  database.
- The `web` crate is responsible for the frontend. It also contains some static
  assets like logos and styling.
- Ignore the `announcers` crate for now.

The other directories, `migrations/` and `schemas/` are used by the database
library. You may need to add to, or modify files in, `schemas/`.

### What, other than Rust, are you using in this project?

- The database is provided by [SurrealDB]. This was mostly just because I wanted
  to play with SurrealDB. This project could absolutely be ran on Postgres,
  maybe even SQLite. But [SurrealQL] is really neat!
- The API is using the [`axum` library]. It also uses [the `surrealdb` crate]
  and [the `surrealdb-migrations` crate] to interact with the database.
- The frontend is built using [the `dioxus` library]. I have adopted a
  component-based setup, apparently, in order to build the site. It works well,
  so I'm sticking with it. The project also uses [TailwindCSS] for styling.

[SurrealDB]: https://surrealdb.com
[SurrealQL]: https://surrealdb.com/docs/surrealql
[`axum` library]: https://docs.rs/axum/latest/axum/
[the `surrealdb` crate]: https://docs.rs/surrealdb/latest/surrealdb/
[the `surrealdb-migrations` crate]: https://docs.rs/surrealdb-migrations/latest/surrealdb_migrations/
[the `dioxus` library]: https://dioxuslabs.com
[TailwindCSS]: https://tailwindcss.componen
