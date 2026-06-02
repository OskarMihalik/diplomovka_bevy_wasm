import { jsonOk, jsonErr, ErrorReason } from "../utils/response"
import { Response } from "express"
import { db } from "../db"
import argon2 from "argon2"
import jwt from "jsonwebtoken"
import { AuthRequest } from "../middleware/auth"

const JWT_SECRET = process.env.JWT_SECRET || "secret"
const PEPPER_SECRET = Buffer.from("mG7JFCuK1/wyoyyaQ1N9lQ")
const ARGON2_HASH_OPTIONS = {
  secret: PEPPER_SECRET,
  type: argon2.argon2id,
  memoryCost: 19000,
  timeCost: 2,
  parallelism: 1,
  hashLength: 32,
} as const

export const login = async (req: AuthRequest, res: Response) => {
  const { email, password } = req.body
  const user = await db
    .selectFrom("User")
    .selectAll()
    .where("email", "=", email)
    .executeTakeFirst()
  if (!user)
    return jsonErr(res, ErrorReason.BadCredentials, "Invalid credentials")

  try {
    const valid = await argon2.verify(
      user.password,
      password,
      ARGON2_HASH_OPTIONS,
    )
    if (!valid)
      return jsonErr(res, ErrorReason.BadCredentials, "Invalid credentials")
  } catch {
    return jsonErr(res, ErrorReason.BadCredentials, "Invalid credentials")
  }

  const token = jwt.sign({ id: user.id, email: user.email }, JWT_SECRET, {
    expiresIn: "1d",
  })
  return jsonOk(res, {
    token,
    id: user.id,
    email: user.email,
    username: user.username,
  })
}

export const register = async (req: AuthRequest, res: Response) => {
  const { email, password, username } = req.body

  if (password.length < 8)
    return jsonErr(
      res,
      ErrorReason.BadRequest,
      "Password must be at least 8 characters long",
    )
  if (username.length < 3)
    return jsonErr(
      res,
      ErrorReason.BadRequest,
      "Username must be at least 3 characters long",
    )

  const hashedPassword = await argon2.hash(password, ARGON2_HASH_OPTIONS)

  try {
    const user = await db
      .insertInto("User")
      .values({
        email,
        username,
        password: hashedPassword,
        salt: "salt_not_used_argon2id_generates_it",
        updated_at: new Date(),
      })
      .returningAll()
      .executeTakeFirstOrThrow()

    const token = jwt.sign({ id: user.id, email: user.email }, JWT_SECRET, {
      expiresIn: "1d",
    })
    return jsonOk(res, {
      token,
      id: user.id,
      email: user.email,
      username: user.username,
    })
  } catch (e: any) {
    return jsonErr(res, ErrorReason.BadRequest, e.message)
  }
}
