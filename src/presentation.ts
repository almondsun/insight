export function changeMetric(hasPreviousSnapshot:boolean,newFollowers:number,lostFollowers:number){
  return hasPreviousSnapshot?`${newFollowers} / ${lostFollowers}`:'—';
}

export function matchesUsername(username:string,search:string){
  return username.toLocaleLowerCase().includes(search.trim().toLocaleLowerCase());
}

export function validInstagramUsername(username:string){
  return /^[A-Za-z0-9._]{1,30}$/.test(username.trim());
}
