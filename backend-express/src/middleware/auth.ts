import { Request, Response, NextFunction } from "express"
import jwt from "jsonwebtoken"
import { db } from "../db"
import dotenv from "dotenv"

dotenv.config({ path: "../.env" })
const JWT_SECRET = process.env.JWT_SECRET || "secret"

export interface AuthRequest extends Request {
  user?: {
    id: number
    email: string
  }
}

export const authenticate = async (
  req: AuthRequest,
  res: Response,
  next: NextFunction,
) => {
  const authHeader = req.headers.authorization
  if (!authHeader) {
    return res.status(401).json({ error: "Unauthorized" })
  }

  const token = authHeader
  try {
    const decoded = jwt.verify(token, JWT_SECRET) as {
      id: number
      email: string
    }
    req.user = decoded
    next()
  } catch (e) {
    return res.status(401).json({ error: "Unauthorized" })
  }
}

export const asyncHandler =
  (fn: Function) => (req: Request, res: Response, next: NextFunction) => {
    Promise.resolve(fn(req as AuthRequest, res, next)).catch(next)
  }
