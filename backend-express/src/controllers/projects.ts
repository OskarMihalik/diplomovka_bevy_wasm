import { jsonOk, jsonEmptyOk, jsonErr, ErrorReason } from "../utils/response"
import { Response } from "express"
import { db } from "../db"
import { AuthRequest } from "../middleware/auth"

const checkAdmin = async (userId: number, projectId: number) => {
  const pUser = await db
    .selectFrom("ProjectUser")
    .where("user_id", "=", userId)
    .where("project_id", "=", projectId)
    .where("is_admin", "=", true)
    .executeTakeFirst()
  return !!pUser
}

export const getProject = async (req: AuthRequest, res: Response) => {
  const projectId = parseInt(req.params.project_id as string)
  const project = await db
    .selectFrom("Project")
    .selectAll()
    .where("id", "=", projectId)
    .executeTakeFirst()
  jsonOk(res, project)
}

export const getProjects = async (req: AuthRequest, res: Response) => {
  // Equivalent of Rust select_projects
  const projects = await db
    .selectFrom("Project as p")
    .innerJoin("ProjectUser as pu", "pu.project_id", "p.id")
    .selectAll("p")
    .where("pu.user_id", "=", req.user!.id)
    .limit(100)
    .offset(0)
    .execute()
  jsonOk(res, projects)
}

export const insertProject = async (req: AuthRequest, res: Response) => {
  const { name, description } = req.body

  // transaction for inserting project and assigning user
  await db.transaction().execute(async (trx) => {
    const project = await trx
      .insertInto("Project")
      .values({
        name,
        description,
        created_by_id: req.user!.id,
        updated_at: new Date(),
      })
      .returning("id")
      .executeTakeFirstOrThrow()

    await trx
      .insertInto("ProjectUser")
      .values({
        project_id: project.id,
        user_id: req.user!.id,
        is_admin: true,
        updated_at: new Date(),
      })
      .execute()
  })

  const projects = await db
    .selectFrom("Project as p")
    .innerJoin("ProjectUser as pu", "pu.project_id", "p.id")
    .selectAll("p")
    .where("pu.user_id", "=", req.user!.id)
    .limit(100)
    .offset(0)
    .execute()

  jsonOk(res, projects)
}

export const updateProject = async (req: AuthRequest, res: Response) => {
  const { id, name, description } = req.body

  if (!(await checkAdmin(req.user!.id, id))) {
    return res
      .status(403)
      .json({ error: "You are not an admin of this project" })
  }

  await db
    .updateTable("Project")
    .set({ name, description, updated_at: new Date() })
    .where("id", "=", id)
    .execute()

  jsonEmptyOk(res)
}

export const deleteProject = async (req: AuthRequest, res: Response) => {
  const projectId = parseInt(req.params.project_id as string)

  if (!(await checkAdmin(req.user!.id, projectId))) {
    return jsonErr(
      res,
      ErrorReason.Unauthorized,
      "You are not an admin of this project",
    )
  }

  await db.deleteFrom("Project").where("id", "=", projectId).execute()
  jsonEmptyOk(res)
}
