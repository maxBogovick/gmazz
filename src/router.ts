import { createRouter, createWebHistory } from 'vue-router';
import NotebookFeed from './views/NotebookFeed.vue';
import SingleNote from './views/SingleNote.vue';
import CreateNote from './views/CreateNote.vue';
import AdminPanel from './views/AdminPanel.vue';

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
  {
    path: '/edit/:id',
    name: 'edit',
    component: CreateNote,
  },
  {
    path: '/admin',
    name: 'admin',
    component: AdminPanel,
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
