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

# bevy app benchmark

F9 runs a 10 s benchmark. The camera does exactly one full orbit, so runs are comparable across models. It then prints a full report:
resolution and frame count
average fps, 1% low and 0.1% low fps
frame time average, min, p50, p95, p99, max and standard deviation
the model numbers: mesh instances, unique meshes, vertices, triangles, materials, textures with memory, entities
