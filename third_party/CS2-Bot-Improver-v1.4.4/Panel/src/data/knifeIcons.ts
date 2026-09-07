// Eagerly bundle every knife icon as a URL, keyed by its subclass id.
const modules = import.meta.glob("../assets/icons/*.png", {
  eager: true,
  query: "?url",
  import: "default",
});

export type KnifeIcon = { id: number; url: string };

export const KNIFE_ICONS: KnifeIcon[] = Object.entries(modules)
  .map(([path, url]) => {
    const m = path.match(/(\d+)\.png$/);
    return { id: m ? parseInt(m[1], 10) : -1, url: url as string };
  })
  .filter((k) => k.id >= 0)
  .sort((a, b) => a.id - b.id);
