/*
  Warnings:

  - You are about to alter the column `position_x` on the `Tag` table. The data in that column could be lost. The data in that column will be cast from `DoublePrecision` to `Real`.
  - You are about to alter the column `position_y` on the `Tag` table. The data in that column could be lost. The data in that column will be cast from `DoublePrecision` to `Real`.
  - You are about to alter the column `position_z` on the `Tag` table. The data in that column could be lost. The data in that column will be cast from `DoublePrecision` to `Real`.

*/
-- AlterTable
ALTER TABLE "Tag" ALTER COLUMN "position_x" SET DATA TYPE REAL,
ALTER COLUMN "position_y" SET DATA TYPE REAL,
ALTER COLUMN "position_z" SET DATA TYPE REAL;
