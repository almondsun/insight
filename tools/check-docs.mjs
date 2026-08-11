import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, extname, join, relative, resolve, sep } from "node:path";

const root=resolve(import.meta.dirname,"..");
const markdown=[];
for(const name of readdirSync(root))if(extname(name)===".md")markdown.push(join(root,name));
function collect(directory){
  for(const name of readdirSync(directory)){
    const path=join(directory,name),stat=statSync(path);
    if(stat.isDirectory())collect(path);else if(extname(name)===".md")markdown.push(path);
  }
}
collect(join(root,"docs"));

const failures=[];
const anchors=new Map();
function slug(value){return value.toLowerCase().trim().replace(/<[^>]+>/g,"").replace(/[^\p{L}\p{N}\s_-]/gu,"").replace(/\s+/g,"-");}
function fileAnchors(path){
  if(anchors.has(path))return anchors.get(path);
  const found=new Set();
  for(const line of readFileSync(path,"utf8").split(/\r?\n/)){
    const heading=line.match(/^#{1,6}\s+(.+?)\s*#*$/);
    if(heading)found.add(slug(heading[1]));
  }
  anchors.set(path,found);return found;
}

for(const source of markdown){
  const body=readFileSync(source,"utf8");
  for(const match of body.matchAll(/!?\[[^\]]*\]\((<[^>]+>|[^)\s]+)(?:\s+"[^"]*")?\)/g)){
    let target=match[1].replace(/^<|>$/g,"");
    if(/^(https?:|mailto:|#)/.test(target)){
      if(target.startsWith("#")&&!fileAnchors(source).has(target.slice(1)))failures.push(`${relative(root,source)}: missing anchor ${target}`);
      continue;
    }
    const [pathPart,fragment]=target.split("#",2);
    const destination=resolve(dirname(source),decodeURIComponent(pathPart));
    if((destination!==root&&!destination.startsWith(`${root}${sep}`))||!existsSync(destination)){failures.push(`${relative(root,source)}: missing ${target}`);continue;}
    if(fragment&&extname(destination)===".md"&&!fileAnchors(destination).has(fragment))failures.push(`${relative(root,source)}: missing anchor ${target}`);
  }
  if(!source.includes(`${join("docs","releases")}`)&&body.includes("github.com/almondsun/insight"))failures.push(`${relative(root,source)}: obsolete canonical repository URL`);
}

if(failures.length){console.error(failures.join("\n"));process.exit(1);}
console.log(`Checked ${markdown.length} Markdown files.`);
