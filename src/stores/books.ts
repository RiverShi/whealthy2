import { defineStore } from "pinia";
import { ref } from "vue";
import { bookApi, type Book } from "@/api/books";

export const useBookStore = defineStore("books", () => {
  const books = ref<Book[]>([]);
  const loading = ref(false);

  async function fetchBooks() {
    loading.value = true;
    try {
      books.value = await bookApi.list();
    } finally {
      loading.value = false;
    }
  }

  async function createBook(name: string) {
    const book = await bookApi.create(name);
    books.value.unshift(book);
    return book;
  }

  async function updateBook(id: string, name: string) {
    const updated = await bookApi.update(id, name);
    const idx = books.value.findIndex((b) => b.id === id);
    if (idx !== -1) books.value[idx] = updated;
    return updated;
  }

  async function archiveBook(id: string) {
    await bookApi.archive(id);
    await fetchBooks();
  }

  async function deleteBook(id: string) {
    await bookApi.remove(id);
    books.value = books.value.filter((b) => b.id !== id);
  }

  return { books, loading, fetchBooks, createBook, updateBook, archiveBook, deleteBook };
});
