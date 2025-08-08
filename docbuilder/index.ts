import { consola } from "consola"
import { existsSync, mkdirSync, readdirSync, readFileSync, rmdirSync, rmSync, writeFileSync, statSync } from "fs"
import { join } from "path"
import { cwd } from "process"
import yaml from "js-yaml"

export type BuildResult = {
    success: boolean
    message?: string
    files?: any[]
    dir_name?: string
    dir_content?: string
}

function buildNav(currentPath: string, relativePath: string = ""): any[] {
    const entries = readdirSync(currentPath, { withFileTypes: true })
    const navEntries: any[] = []

    for (const entry of entries) {
        const entryPath = join(currentPath, entry.name)
        const entryRelPath = join(relativePath, entry.name)

        if (entry.isDirectory()) {
            const childNav = buildNav(entryPath, entryRelPath)
            if (childNav.length > 0) {
                navEntries.push({ [entry.name]: childNav })
            }
        } else if (entry.name.endsWith(".md")) {
            const title = entry.name.replace(/\.md$/, "")
            if (title === "index") {
                navEntries.unshift({ [entryRelPath.split("/").slice(-2, -1)[0] as any ?? "Index"]: entryRelPath.replace(/\\/g, "/") })
                consola.info(`Bound explicit name alias "${entryRelPath}" as folder index`)
            } else {
                navEntries.push({ [title]: entryRelPath.replace(/\\/g, "/") })
                consola.info(`Bound file "${title}" to ${entryRelPath}`)
            }
        }
    }

    return navEntries
}

let files_added = 0
let files_removed = 0
let characters_written = 0

consola.box("Velvet DocBuilder\nusing MkDocs & material-mkdocs")
consola.start("Starting build process...\n")
const build_step_dir = join(cwd(), "build_steps")
const src_path = join(cwd(), "../src")
const start_build_time = Date.now()
for (const build_step_file of readdirSync(build_step_dir)) {
    const start_sub_build_time = Date.now()
    const build_step_file_import = await import(join(build_step_dir, build_step_file))
    const data = build_step_file_import.default
    consola.start(`Building ${data.name}...`)
    const result = await data.build_executor(src_path)
    console.log(result)
    if (!result.success) {
        consola.error(`Failed to build step "${data.name}": ${result.message}`)
        process.exit(-1)
    }

    // Build target dir
    const target_path = join(cwd(), "../docs/docs", data.target)
    consola.info(`- ${target_path}`)
    files_removed += 1
    rmSync(target_path, { recursive: true, force: true })
    consola.info(`+ ${target_path}`)
    files_added += 1
    mkdirSync(target_path)

    // Build dir
    consola.info(`+ ${join(target_path, result.dir_name)}`)
    files_added += 1
    mkdirSync(join(target_path, result.dir_name))
    consola.info(`+ ${join(target_path, result.dir_name, `index.md`)}`)
    files_added += 1
    // writeFileSync(join(target_path, result.dir_name, `index.md`), result.dir_content)
    characters_written += result.dir_content.length

    // Build sub-files
    for (const [sub_folder, sub_folder_data] of Object.entries(result.files)) {
        const sub_dir_path = join(target_path, sub_folder)
        mkdirSync(sub_dir_path, { recursive: true })
        consola.info(`+ ${sub_dir_path}`)

        for (const sub_file of (sub_folder_data as any).files as any) {
            const full_path = join(sub_dir_path, sub_file.name)
            consola.info(`+ ${full_path}`)
            files_added += 1
            writeFileSync(full_path, sub_file.content)
            characters_written += sub_file.content.length
        }

        // Write index.md (if provided) inside each sub_folder
        if ((sub_folder_data as any).dir_content) {
            const index_path = join(sub_dir_path, "index.md")
            writeFileSync(index_path, (sub_folder_data as any).dir_content)
            consola.info(`+ ${index_path}`)
            files_added += 1
            characters_written += (sub_folder_data as any).dir_content.length
        }
    }
    consola.success(`Finished building ${data.name} in ${Date.now() - start_sub_build_time}ms\n`)
}

consola.start(`Rebuilding mkdocs navbar config...`)
const mkdocsPath = join(cwd(), "../docs/mkdocs.yml")
const mkdocsRaw = readFileSync(mkdocsPath, "utf-8")
const mkdocsConfig = yaml.load(mkdocsRaw) as any

const docsRoot = join(cwd(), "../docs/docs")

const newNav = buildNav(docsRoot)

mkdocsConfig.nav = newNav

const updatedYaml = yaml.dump(mkdocsConfig, { lineWidth: 1000 })
characters_written += updatedYaml.length
writeFileSync(mkdocsPath, updatedYaml, "utf-8")

consola.success("Updated mkdocs.yml nav section\n")
consola.box(`Finished building documentation\n\n${files_added} file(s) added\n${files_removed} file(s) removed\n${characters_written} characters written\n\nTook ${Date.now() - start_build_time}ms`)
