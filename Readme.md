# how to run

## db

`docker compose up`

## backend

`cargo run -p backend`

## frontend

native:
`cargo run -p bevy_diplomovka`

wasm:

```
cd bevy_diplomovka
trunk serve
```

# db schema

when first staring db:

```
cd prisma
npm ci --legacy-peer-deps
npx prisma db push
```

it is managed by prisma
if you made changes to schema.prisma update db by:

```
npx prisma migrate dev --name init
```
