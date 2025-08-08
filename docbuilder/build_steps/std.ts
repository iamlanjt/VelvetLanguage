// Building the standard lib documentation
// %docs -> Standard Library...

import { join } from "path"
import { existsSync, readdirSync } from "fs"
import consola from "consola"
import type { BuildResult } from ".."

function type_to_rep(t: string) {
    switch (t) {
        case "wc": {
            return "*"
            break;
        }
        default: {
            return t;
            break;
        }
    }
}

export default {
    name: "Standard Library Documentation",
    target: "./Standard Library",
    build_executor: async (src_path: string): Promise<BuildResult> => {
        const ret = {
            success: true,
            files: {} as any,
            dir_name: "?",
            dir_content: "?",
        }
        const externals_dir = join(src_path, "stdlib_comp/externals")
        if (!existsSync(externals_dir)) {
            return {
                success: false,
                message: "stdlib_comp/externals dir does not exist"
            }
        }

        for (const sub_module of readdirSync(externals_dir)) {
            const toml_path = join(externals_dir, sub_module, "meta.toml")
            if (!existsSync(toml_path)) {
                return {
                    success: false,
                    message: `${toml_path} could not be resolved or does not exist.`
                }
            }

            const toml_content = await import(toml_path)
            ret.dir_name = toml_content.name
            ret.dir_content = `# ${ret.dir_name}\n${toml_content.desc}`
            let file_buff = {
                files: [] as any,
                dir_name: "",
                dir_content: ""
            }
            for (const submod of toml_content.submod) {
                console.log(`${toml_content.name} ${submod.name}`)
                const arg_string = submod.args.map((f: any) => {
                    return `${f.name} as ${type_to_rep(f.type)}`
                })
                file_buff.dir_content += `\n\n${toml_content.name}::[${submod.name}](${submod.name}.md) -> \`${type_to_rep(submod.return_type ?? "void")}\``
                file_buff.files.push({
                    name: `${submod.name}.md`,
                    content: `## Function **${submod.name}**\n\`\`\`\n${submod.name}(${arg_string.join(", ")}) -> ${type_to_rep(submod.return_type ?? "void")}\n\`\`\`\n\n${submod.desc}\n\n## Errors\n${submod.errs ?? "n/a"}\n\n<sub>internal rust::${submod.name_mirror ?? submod.name}</sub>`
                })
            }
            ret.files[ret.dir_name] = file_buff
        }

        return ret
    }
}