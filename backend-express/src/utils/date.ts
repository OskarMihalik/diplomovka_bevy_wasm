export const toPrimitiveDateTime = (d: Date | string | null | undefined): number[] | null => {
    if (!d) return null;
    const date = new Date(d);
    
    // pg driver returns timestamp without time zone as local time dates.
    // To match the exact digits in the database (which PrimitiveDateTime maps to), we extract local parts.
    const year = date.getFullYear();
    const month = date.getMonth();
    const day = date.getDate();
    
    // ordinal day calculation
    // JS Date month is 0-indexed.
    const start = new Date(year, 0, 0);
    const target = new Date(year, month, day);
    const diff = target.getTime() - start.getTime();
    const ordinalDay = Math.floor(diff / (1000 * 60 * 60 * 24));
    
    return [
        year,
        ordinalDay,
        date.getHours(),
        date.getMinutes(),
        date.getSeconds(),
        date.getMilliseconds() * 1000000
    ];
};
