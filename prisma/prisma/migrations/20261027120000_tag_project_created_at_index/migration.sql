-- CreateIndex
CREATE INDEX "Tag_project_id_created_at_idx" ON "Tag"("project_id", "created_at" DESC);
