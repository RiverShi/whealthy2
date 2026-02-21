import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { bookApi, type Book } from "@/api/books";

export const useBookStore = defineStore("books", () => {
  const books = ref<Book[]>([]);
  const loading = ref(false);
  const activeBookId = ref<string | null>(null);

  const activeBook = computed(() =>
    books.value.find((b) => b.id === activeBookId.value) ?? null
  );

  async function fetchBooks() {
    loading.value = true;
    try {
      books.value = await bookApi.list();
      // 默认激活第一本账本
      if (!activeBookId.value && books.value.length) {
        activeBookId.value = books.value[0].id;
      }
    } finally {
      loading.value = false;
    }
  }

  function setActiveBook(id: string) {
    activeBookId.value = id;
  }

  async function createBook(name: string) {
    const book = await bookApi.create(name);
    books.value.unshift(book);
    if (!activeBookId.value) activeBookId.value = book.id;
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
    if (activeBookId.value === id) {
      activeBookId.value = books.value[0]?.id ?? null;
    }
  }

  async function deleteBook(id: string) {
    await bookApi.remove(id);
    books.value = books.value.filter((b) => b.id !== id);
    if (activeBookId.value === id) {
      activeBookId.value = books.value[0]?.id ?? null;
    }
  }

  return { books, loading, activeBookId, activeBook, fetchBooks, setActiveBook, createBook, updateBook, archiveBook, deleteBook };
});
