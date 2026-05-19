Date.prototype.toJSON = function() {
    const year = this.getFullYear();
    const month = this.getMonth();
    const day = this.getDate();
    
    // NOTE: start is end of previous year (month 0, day 0 is Dec 31 of previous year)
    const start = new Date(year, 0, 0);
    const target = new Date(year, month, day);
    const diff = target.getTime() - start.getTime();
    const ordinalDay = Math.round(diff / (1000 * 60 * 60 * 24));
    
    return [
        year,
        ordinalDay,
        this.getHours(),
        this.getMinutes(),
        this.getSeconds(),
        this.getMilliseconds() * 1000000
    ];
};
console.log(JSON.stringify({ created_at: new Date('2026-05-14 09:22:32.105') }));
