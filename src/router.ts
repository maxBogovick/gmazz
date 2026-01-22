import { createRouter, createWebHistory } from 'vue-router';
import NotebookFeed from './views/NotebookFeed.vue';
import SingleNote from './views/SingleNote.vue';
import CreateNote from './views/CreateNote.vue';

const routes = [
  {
    path: '/',
    name: 'feed',
    component: NotebookFeed,
  },
  {
    path: '/note/:id',
    name: 'note',
    component: SingleNote,
  },
  {
    path: '/create',
    name: 'create',
    component: CreateNote,
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
