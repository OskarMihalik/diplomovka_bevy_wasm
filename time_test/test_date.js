function toPrimitiveDateTime(d) {
    if (!d) return null;
    const date = new Date(d);
    
    // We want the local parts if pg parsed it as local, or UTC?
    // Let's use getUTCFullYear if the db timezone is UTC or just test Date values.
    
    const year = date.getUTCFullYear();
    const month = date.getUTCMonth();
    const dayOfMonth = date.getUTCDate();
    
    // ordinal day
    const start = new Date(Date.UTC(year, 0, 0));
    const diff = date.getTime() - start.getTime();
    const ordinalDay = Math.floor(diff / (1000 * 60 * 60 * 24));
    
    return [
        year,
        ordinalDay,
        date.getUTCHours(),
        date.getUTCMinutes(),
        date.getUTCSeconds(),
        date.getUTCMilliseconds() * 1000000
    ];
}

console.log(toPrimitiveDateTime(new Date('2026-05-14T09:22:32.105Z')));
