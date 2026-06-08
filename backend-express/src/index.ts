import express from "express"
import cors from "cors"
import dotenv from "dotenv"
;(Date.prototype as any).toJSON = function () {
  const year = this.getFullYear()
  const month = this.getMonth()
  const day = this.getDate()

  const start = new Date(year, 0, 0)
  const target = new Date(year, month, day)
  const diff = target.getTime() - start.getTime()
  const ordinalDay = Math.round(diff / (1000 * 60 * 60 * 24))

  return [
    year,
    ordinalDay,
    this.getHours(),
    this.getMinutes(),
    this.getSeconds(),
    this.getMilliseconds() * 1000000,
  ]
}
import path from "path"
import { authenticate, asyncHandler } from "./middleware/auth"
import { login, register } from "./controllers/auth"
import {
  getProject,
  getProjects,
  insertProject,
  updateProject,
  deleteProject,
} from "./controllers/projects"
import { getModel, getModels, uploadModel, upload } from "./controllers/models"
import { getTags, insertTag, updateTag, deleteTag } from "./controllers/tags"
import { getTagMessages, createTagMessage } from "./controllers/tag_messages"
import {
  getStatuses,
  createStatus,
  updateStatus,
  deleteStatus,
} from "./controllers/statuses"
import {
  getUsers,
  addUserToProject,
  getUsersInProject,
} from "./controllers/users"

dotenv.config({ path: "../.env" })

const app = express()
app.use(cors())
app.use(express.json({ limit: "1gb" }))

const ASSETS_MODELS = path.join(__dirname, "../../backend/assets/models")
const ASSETS_ATTACHMENTS = path.join(
  __dirname,
  "../../backend/assets/attachments",
)

app.use("/assets/model", express.static(ASSETS_MODELS))
app.use("/assets/attachments", express.static(ASSETS_ATTACHMENTS))

// Public Routes
app.post("/login", asyncHandler(login))
app.post("/register", asyncHandler(register))

// Auth Middleware for below routes
app.use(authenticate)

// Tags
app.post("/tags", asyncHandler(insertTag))
app.patch("/tags", asyncHandler(updateTag))
app.get("/tags/:model_id", asyncHandler(getTags))
app.delete("/tag/:tag_id", asyncHandler(deleteTag))

// Tag Messages
app.get("/tag_message/:tag_id", asyncHandler(getTagMessages))
app.post("/tag_message", asyncHandler(createTagMessage))

// Models
app.get("/model/:model_id", asyncHandler(getModel))
app.get("/models/:project_id", asyncHandler(getModels))
app.post(
  "/model/:project_id/:model_name",
  upload.single("file"),
  asyncHandler(uploadModel),
)

// Projects
app.get("/project/:project_id", asyncHandler(getProject))
app.delete("/project/:project_id", asyncHandler(deleteProject))
app.get("/project", asyncHandler(getProjects))
app.patch("/project", asyncHandler(updateProject))
app.post("/project", asyncHandler(insertProject))

// Users
app.post("/users", asyncHandler(getUsers))
app.post("/project_user", asyncHandler(addUserToProject))
app.get("/users/:project_id", asyncHandler(getUsersInProject))

// Statuses
app.get("/statuses/:project_id", asyncHandler(getStatuses))
app.put("/status", asyncHandler(createStatus))
app.post("/status", asyncHandler(updateStatus))
app.delete("/status/:status_id", asyncHandler(deleteStatus))

const port = process.env.BACKEND_PORT || 3000
app.listen(port, () => {
  console.log(`Backend Express running on port ${port}`)
})
