const who: string = 'world';

let heading = document.createElement('h1');
heading.textContent = `Hello ${who}!`;

document.body.appendChild(heading);
