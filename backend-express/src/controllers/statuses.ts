import { jsonOk, jsonEmptyOk, jsonErr, ErrorReason } from "../utils/response"
import { Response } from "express"
import { db } from "../db"
import { AuthRequest } from "../middleware/auth"

export const getStatuses = async (req: AuthRequest, res: Response) => {
  const projectId = parseInt(req.params.project_id as string)
  const statuses = await db
    .selectFrom("Status")
    .selectAll()
    .where("project_id", "=", projectId)
    .limit(100)
    .offset(0)
    .execute()
  jsonOk(res, statuses)
}

export const createStatus = async (req: AuthRequest, res: Response) => {
  const { title, color_r, color_g, color_b, project_id, shape } = req.body
  await db
    .insertInto("Status")
    .values({
      title,
      color_r,
      color_g,
      color_b,
      project_id,
      shape: shape || "Cuboid",
      updated_at: new Date(),
    })
    .execute()

  jsonEmptyOk(res)
}

export const updateStatus = async (req: AuthRequest, res: Response) => {
  const { id, title, color_r, color_g, color_b, project_id, shape } = req.body
  await db
    .updateTable("Status")
    .set({
      title,
      color_r,
      color_g,
      color_b,
      project_id,
      shape,
      updated_at: new Date(),
    })
    .where("id", "=", id)
    .execute()
  jsonEmptyOk(res)
}

export const deleteStatus = async (req: AuthRequest, res: Response) => {
  const statusId = parseInt(req.params.status_id as string)
  await db.deleteFrom("Status").where("id", "=", statusId).execute()
  jsonEmptyOk(res)
}
