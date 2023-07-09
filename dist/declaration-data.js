// Replacement version of declaration-data.js

async function req(url, body, default_, json_or_text = "json") {
    let response;
    try {
        response = await fetch(url, {
            method: "POST",
            body: JSON.stringify(body),
            headers: {
                "Content-Type": "application/json"
            }
        });
    } catch (exc) {
        console.error("Error in req: ", exc);
        return default_;
    }

    let data;
    try {
        data = await response.text();
    } catch (exc) {
        console.error("Error in req:", data);
        return default_;
    }

    if (json_or_text === "json") {
        try {
            data = JSON.parse(data);
        } catch (exc) {
            console.error("Error in req, got message:", data);
            return default_;
        }
    }

    return data;
}

export class DeclarationDataCenter {
    static singleton = null;

    constructor() { }

    static async init() {
        DeclarationDataCenter.singleton = new DeclarationDataCenter();
        return DeclarationDataCenter.singleton;
    }

    /**
   * Search for a declaration.
   * @returns {Array<any>}
   */
    async search(pattern, strict = true, allowedKinds = undefined, maxResults = undefined) {
        // if (!pattern) {
        //     return [];
        // }
        // if (strict) {
        //     let decl = this.declarationData.declarations[pattern];
        //     return decl ? [decl] : [];
        // } else {
        //     return getMatches(this.declarationData.declarations, pattern, allowedKinds, maxResults);
        // } 

        return await req("/search_decl", {
            pattern,
            strict,
            allowedKinds,
            maxResults
        }, []);
    }

    // TODO: some of these are probably small enough that they can be done client-side?
    /**
   * Search for all instances of a certain typeclass
   * @returns {Array<String>}
   */
    async instancesForClass(className) {
        console.error("instancesForClass not implemented yet");

        return await req("/instances_for_class", className, []);
    }

    /**
     * Search for all instances that involve a certain type
     * @returns {Array<String>}
     */
    async instancesForType(typeName) {
        console.error("instancesForType not implemented yet");

        return await req("/instances_for_type", typeName, []);
    }

    /**
     * Get all instances and their links for the given typeclasses 
     * @returns {Array<Array<Object<String, String>>>}
     */
    async annotateInstances(names) {
        return await req("/annotate_instances", {
            names
        }, {});
    }

    /**
     * Get all instances and their links for the given types
     * @returns {Array<Array<Object<String, String>>>}
     */
    async annotateInstancesFor(names) {
        return await req("/annotate_instances_for", {
            names
        }, {});
    }

    /**
     * Analogous to Lean declNameToLink
     * @returns {String}
     */
    async declNameToLink(declName) {
        // return this.declarationData.declarations[declName].docLink;
        console.error("declNameToLink not implemented yet");

        // try {
        //     let response = await fetch("/decl_name_to_link", {
        //         method: "POST",
        //         body: declName,
        //     });

        //     let data = await response.text();
        //     return data;
        // } catch (exc) {
        //     console.error("Error in declNameToLink: ", exc);
        //     return "";
        // }

        return await req("/decl_name_to_link", declName, "");
    }

    /**
     * Find all modules that imported the given one.
     * @returns {Array<String>}
     */
    async moduleImportedBy(moduleName) {
        // return this.declarationData.importedBy[moduleName];
        console.error("moduleImportedBy not implemented yet");

        //     try {
        //         let response = await fetch("/module_imported_by", {
        //             method: "POST",
        //             body: moduleName,
        //         });

        //         let data = await response.json();
        //         return data;
        //     } catch (exc) {
        //         console.error("Error in moduleImportedBy: ", exc);
        //         return [];
        //     }

        return await req("/module_imported_by", moduleName, []);
    }

    /**
     * Analogous to Lean moduleNameToLink
     * @returns {String}
     */
    async moduleNameToLink(moduleName) {
        // return this.declarationData.modules[moduleName];
        console.error("moduleNameToLink not implemented yet");

        // try {
        //     let response = await fetch("/module_name_to_link", {
        //         method: "POST",
        //         body: moduleName,
        //     });

        //     let data = await response.text();
        //     return data;
        // } catch (exc) {
        //     console.error("Error in moduleNameToLink: ", exc);
        //     return "";
        // }

        return await req("/module_name_to_link", moduleName, "");
    }

    async linkedImportedBy(moduleName) {
        return await req("/linked_imported_by", moduleName, []);
    }
}