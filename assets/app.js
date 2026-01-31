const API_BASE = '/api';
const app = document.getElementById('main-content');
const modal = document.getElementById('definition-modal');
let currentBook = null;
let currentRendition = null;
let currentWord = null;
let currentContext = null;

async function fetchAPI(endpoint, options = {}) {
    const res = await fetch(`${API_BASE}${endpoint}`, options);
    if (!res.ok) throw new Error(`API Error: ${res.statusText}`);
    return res.json();
}

async function showLibrary() {
    try {
        const books = await fetchAPI('/books');
        app.innerHTML = `
            <h2>Library</h2>
            <div class="book-grid">
                ${books.map(book => `
                    <div class="book-card" onclick="openBook('${book.path}')">
                        <h3>${book.name}</h3>
                        <p>${book.path.split('.').pop().toUpperCase()}</p>
                    </div>
                `).join('')}
            </div>
        `;
    } catch (e) {
        app.innerHTML = `<p class="error">Failed to load library: ${e.message}</p>`;
    }
}

async function openBook(path) {
    currentBook = path;
    const ext = path.split('.').pop().toLowerCase();

    app.innerHTML = `<div id="reader-loading">Loading...</div>`;

    if (ext === 'epub') {
        openEpub(path);
    } else if (ext === 'txt') {
        openTxt(path);
    } else {
        app.innerHTML = `<p>Unsupported format for interactive reading. <a href="/api/books/${path}" target="_blank">Download/View Raw</a></p>`;
    }
}

async function openTxt(path) {
    try {
        const res = await fetch(`${API_BASE}/books/${path}`);
        const text = await res.text();

        // Wrap words in spans
        const html = text.split(/\n/).map(para => {
            return `<p>${para.split(/\s+/).map(word => {
                // simple cleanup
                const cleanWord = word.replace(/[^\w\u00C0-\u00FF]/g, '');
                if (!cleanWord) return word;
                return `<span class="clickable-word" onclick="handleWordClick('${cleanWord}', this)">${word}</span>`;
            }).join(' ')}</p>`;
        }).join('');

        app.innerHTML = `
            <div class="reader-container">
                <h2>${path}</h2>
                <div class="text-content">${html}</div>
            </div>
        `;
    } catch (e) {
        app.innerHTML = `<p>Error loading book: ${e.message}</p>`;
    }
}

function openEpub(path) {
    app.innerHTML = `
        <div id="epub-viewer"></div>
        <div style="text-align: center; margin-top: 10px;">
            <button onclick="currentRendition.prev()">Previous</button>
            <button onclick="currentRendition.next()">Next</button>
            <p><em>Select text to look it up</em></p>
        </div>
    `;

    const book = ePub(`${API_BASE}/books/${path}`);
    currentRendition = book.renderTo("epub-viewer", {
        width: "100%",
        height: "100%",
        flow: "scrolled-doc"
    });

    currentRendition.display();

    currentRendition.on("selected", (cfiRange, contents) => {
        book.getRange(cfiRange).then(range => {
            const word = range.toString().trim();
            if (word) {
                // Get some context (approximate)
                const text = contents.document.body.innerText;
                // Context extraction is hard from just range, using the word for now
                lookupWord(word);
                // clear selection
                contents.window.getSelection().removeAllRanges();
            }
        });
    });
}

async function handleWordClick(word, element) {
    currentContext = element.closest('p')?.innerText;
    lookupWord(word);
}

async function lookupWord(word) {
    currentWord = word;
    const modalWord = document.getElementById('modal-word');
    const modalDefs = document.getElementById('modal-definitions');

    modalWord.innerText = word;
    modalDefs.innerHTML = 'Loading...';
    modal.classList.remove('hidden');

    try {
        const entries = await fetchAPI(`/lookup/${word}`);
        if (entries.length === 0) {
            modalDefs.innerHTML = '<p>No definition found.</p>';
            return;
        }

        modalDefs.innerHTML = entries.map(entry => `
            <div class="definition-entry">
                <p><strong>${entry.word}</strong> <span class="pos-tag">${entry.pos}</span></p>
                <ul>
                    ${entry.senses.map(sense => `<li>${sense.raw_glosses ? sense.raw_glosses.join(', ') : sense.glosses ? sense.glosses.join(', ') : 'No definition'}</li>`).join('')}
                </ul>
            </div>
        `).join('');
    } catch (e) {
        modalDefs.innerHTML = `<p>Error: ${e.message}</p>`;
    }
}

function closeModal() {
    modal.classList.add('hidden');
}

document.getElementById('add-flashcard-btn').onclick = async () => {
    if (!currentWord) return;

    // Get the first definition as default
    const firstDef = document.querySelector('.definition-entry ul li')?.innerText || 'No definition selected';

    try {
        await fetchAPI('/flashcards', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                word: currentWord,
                definition: firstDef,
                context: currentContext,
                source: currentBook
            })
        });
        alert('Added to flashcards!');
        closeModal();
    } catch (e) {
        alert('Failed to add flashcard: ' + e.message);
    }
};

async function showFlashcards() {
    try {
        const cards = await fetchAPI('/flashcards');
        app.innerHTML = `
            <h2>My Flashcards</h2>
            <div class="flashcard-list">
                ${cards.length === 0 ? '<p>No flashcards yet.</p>' : ''}
                ${cards.map(card => `
                    <div class="definition-entry">
                        <h3>${card.word}</h3>
                        <p>${card.definition}</p>
                        ${card.context ? `<p><em>"${card.context}"</em></p>` : ''}
                        <small>From: ${card.source || 'Unknown'}</small>
                    </div>
                `).join('')}
            </div>
        `;
    } catch (e) {
        app.innerHTML = `<p>Error loading flashcards: ${e.message}</p>`;
    }
}

// Start app
showLibrary();
