const replacRegex = new RegExp(/[-\s]+/g);
const idPartRegex = new RegExp(/([a-zA-Z0-9_-\s]+)/g);

export function makeId(value: string) {
    value = value.normalize("NFKC");
    const idPartsMatch = value.matchAll(idPartRegex);

    let id = "";

    for (const match of idPartsMatch) {
        for (let i = 1; i < match.length; ++i) {
            id += match[i];
        }
    }

    return id.toLowerCase().replace(replacRegex, "_");
}
